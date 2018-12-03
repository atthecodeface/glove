/*a Copyright */
/*a Doc

convert -depth 8 -size 640x480 a.gray a.jpg

 */
/*a Includes */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h> // for usleep
#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>

#include <signal.h>
#include <strings.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <pthread.h>

#include "capture_device.h"
#include "server.h"
#include "highlights.h"

/*a Types */
enum {
    dt_action_flush=1,
    dt_action_capture=2,
    dt_action_accumulate=4,
    dt_action_track_highlights=8,
};
struct device_thread {
    pthread_t pthread;
    pthread_mutex_t m;    
    int started;
    int halt;
    int action;
    int action_state;
    int args[4];
    struct highlight_set *hs;
};
struct highlights_server {
    struct capture_device *dev;
    struct server_skt     *skt;
    char *filename;
    pthread_t srv_thread;
    struct device_thread dev_thread;
};

/*a Capture callback functions */
/*f find_highlights */
static
void threshold_capture_callback(void *handle, const unsigned char *buffer, int width, int height )
{
    int *result = (int *)handle;
    int i, n;
    for (i=n=0; i<width*height*2; i+=2) {
        if (buffer[i]>*result) n++;
    }
    *result = n;
}

/*f accumulate - accumulate into an int * buffer */
void accumulate(void *handle, const unsigned char *buffer, int width, int height)
{
    int **acc = (int **)handle;
    int i;
    if (!(*acc)) {
        *acc = (int *)malloc(width*height*sizeof(int));
        for (i=0; i<width*height; i++) {
            (*acc)[i] = 0;
        }
    }
    for (i=0; i<width*height; i++) {
        (*acc)[i] += buffer[i*2];
    }
}

/*f dump_acc_buffer */
static
void dump_acc_buffer(const char *filename, int *acc_buffer, int frames)
{
    FILE *f = fopen(filename,"w");
    int i;
    int max;
    max = 0;
    for (i=0; i<640*480; i++) {
        char b = acc_buffer[i]/frames;
        if (acc_buffer[i]>max) max=acc_buffer[i];
        fwrite(&b, 1, 1, f);
    }
    fclose(f);
    fprintf(stderr,"Max %d %d\n",max,max/frames);
}

/*a Device thread functions */
/*f device_start_action */
static
void device_start_action(struct highlights_server *hs, int action)
{
    pthread_mutex_lock(&hs->dev_thread.m);
    hs->dev_thread.action = action;
    pthread_mutex_unlock(&hs->dev_thread.m);
}

/*f device_get_action */
static
int device_get_action(struct highlights_server *hs)
{
    int action;
    pthread_mutex_lock(&hs->dev_thread.m);
    action = hs->dev_thread.action;
    if (action==0) {
        pthread_mutex_unlock(&hs->dev_thread.m);
        usleep(1000);
        return 0;
    }
    hs->dev_thread.action_state = 0;
    hs->dev_thread.action = 0;
    pthread_mutex_unlock(&hs->dev_thread.m);
    return action;
}

/*f device_complete_action */
static
void device_complete_action(struct highlights_server *hs, int rc)
{
    pthread_mutex_lock(&hs->dev_thread.m);
    hs->dev_thread.action_state = rc;
    pthread_mutex_unlock(&hs->dev_thread.m);
}

/*f device_poll_for_action_complete */
static
int device_poll_for_action_complete(struct highlights_server *hs)
{
    while (hs->dev_thread.started) {
        int action, action_state;
        pthread_mutex_lock(&hs->dev_thread.m);
        action       = hs->dev_thread.action;
        action_state = hs->dev_thread.action_state;
        pthread_mutex_unlock(&hs->dev_thread.m);
        if ((action==0) && (action_state!=0)) {
            return action_state;
        }
        usleep(1000);
    }
    return -1;
}

/*f device_thread */
static
void *device_thread(void *handle)
{
    struct highlights_server *hs = (struct highlights_server *)handle;
    while (!hs->dev_thread.halt) {
        int action = device_get_action(hs);
        int rc = 1;
        if (action==0) continue;
        if (action & dt_action_flush) {
            fprintf(stderr, "Flushed %d\n",cd_flush(hs->dev));
            usleep(100000);
            fprintf(stderr, "Flushed %d\n",cd_flush(hs->dev));
        }
        if (action & dt_action_capture) {
            rc = capture_frame(hs->dev, 1000*1000*4, dump_to_file, (void *)hs->filename);
        }
        if (action & dt_action_accumulate) {
            int *acc_buffer = NULL;
            int i, frames;
            frames = hs->dev_thread.args[0];
            for (i=0; i<frames; i++) {
                rc = capture_frame(hs->dev, 1000*1000*4, accumulate, &acc_buffer);
                if (rc<1) break;
            }
            if (acc_buffer) {
                dump_acc_buffer(hs->filename, acc_buffer, frames);
                free(acc_buffer);
            }
        }
        if (action & dt_action_track_highlights) {
            int i, frames;
            frames = hs->dev_thread.args[0];
            for (i=0; i<frames; i++) {
                highlight_set_precapture(hs->dev_thread.hs, 0);
                capture_frame(hs->dev, 1000*1000*4, find_highlights, hs->dev_thread.hs);
                highlight_set_complete(hs->dev_thread.hs);
            }
            rc = 1;
        }
        device_complete_action(hs, rc);
    }
    hs->dev_thread.started = 0;
    return 0;
}

/*f device_halt_thread */
static void device_halt_thread(struct highlights_server *hs)
{
    hs->dev_thread.halt = 1;
}

/*f device_start_thread */
static int device_start_thread(void *handle)
{
    struct highlights_server *hs = (struct highlights_server *)handle;
    pthread_attr_t attr;
    int rc=0;
    hs->dev_thread.halt = 0;
    if (pthread_attr_init(&attr)<0) return -1;
    hs->dev_thread.started = 1;
    if (pthread_mutex_init(&hs->dev_thread.m, NULL)<0) return -1;
    rc = pthread_create(&hs->dev_thread.pthread, &attr, device_thread, (void *)hs);
    pthread_attr_destroy(&attr);
    if (rc<0) {hs->dev_thread.started = 0;}
    return rc;
}

/*a Server callback functions */
/*f data_callback
return <0 on error (close socket)
return 0 if buffer cannot be handled - yet
return number of bytes consumed otherwise
 */
static
int server_data_callback(struct server_skt *skt, void *handle, char *in_buffer, int in_valid) {
    struct highlights_server *hs = (struct highlights_server *)handle;
    struct capture_device *dev = hs->dev;
    int cmd_len;
    char buffer[64];
    if (in_valid<4) return 0;
    for (cmd_len=0; (cmd_len<in_valid) && (in_buffer[cmd_len]>=' '); cmd_len++);
    if (cmd_len>=in_valid) return 0;
    in_buffer[cmd_len] = 0;
    fprintf(stderr,"Command '%s'\n", in_buffer);
    if (!strncmp(in_buffer,"dump",4)) {
        device_start_action(hs, dt_action_capture | dt_action_flush);
        sprintf(buffer,"%d\n", device_poll_for_action_complete(hs));
        server_add_to_send(skt, buffer, strlen(buffer));
    }
    if (!strncmp(in_buffer,"accum",5)) {
        int frames;
        int rc=-2;
        if (sscanf(in_buffer,"accum %d",&frames)==1) {
            hs->dev_thread.args[0] = frames;
            device_start_action(hs, dt_action_accumulate | dt_action_flush);
            rc = device_poll_for_action_complete(hs);
        }
        sprintf(buffer,"%d\n",rc);
        server_add_to_send(skt, buffer, strlen(buffer));
    }
    if (!strncmp(in_buffer,"track",5)) {
        int frames;
        int rc=-2;
        if (sscanf(in_buffer,"track %d",&frames)==1) {
            hs->dev_thread.args[0] = frames;
            device_start_action(hs, dt_action_track_highlights | dt_action_flush);
            rc = device_poll_for_action_complete(hs);
        }
        sprintf(buffer,"%d\n",rc);
        server_add_to_send(skt, buffer, strlen(buffer));
    }
    if (!strncmp(in_buffer,"thresh",6)) {
        cd_flush(dev);
        int rc, frames, result, max_result;
        max_result = 0;
        result = 100;
        rc = -2;
        frames = 1;
        if (sscanf(in_buffer,"thresh %d %d",&result,&frames)>=1) {
            while (frames>0) {
                rc = capture_frame(dev, 1000*1000*4, threshold_capture_callback, &result);
                if (result>max_result) max_result=result;
                if (rc<1) break;
                frames--;
            }
        }
        if (rc>0) {
            sprintf(buffer,"%d\n",max_result);
        } else {
            sprintf(buffer,"%d\n",rc);
        }
        server_add_to_send(skt, buffer, strlen(buffer));
    }
    if (!strncmp(in_buffer,"set",3)) {
        int rc=-2;
        int param, value;
        if (sscanf(in_buffer,"set %d %d",&param,&value)==2) {
            rc = cd_set_parameter(dev, param, value);
        }
        sprintf(buffer,"%d\n",rc);
        server_add_to_send(skt, buffer, strlen(buffer));
    }
    if (!strcmp(in_buffer,"close")) {
        return -1;
    }
    if (!strcmp(in_buffer,"shutdown")) {
        server_halt_thread(hs->skt);
        device_halt_thread(hs);
        return -1;
    }
    return cmd_len+1;
}

/*a Main and signal handler */
struct highlights_server hs;
static void sig_handler(int s) {
    server_halt_thread(hs.skt);
    device_halt_thread(&hs);
}

/*f Main */
int main(int argc, char **argv)
{
    int dev_num, port;
    if (argc!=2) {
        fprintf(stderr, "Usage %s <device>\n", argv[0]);
        return 4;
    }
    dev_num = argv[1][strlen(argv[1])-1];
    if ((dev_num>='0') && (dev_num<='9')) dev_num=(dev_num-'0');

    port = 1234 + dev_num;
    hs.filename = malloc(256);
    sprintf(hs.filename, "a%d.gray", dev_num);
    
    hs.dev = create_device(argv[1]);
    struct highlight_set *highlights = create_highlight_set(1);
    hs.skt = server_create(port);
    if (!hs.skt) {
        fprintf(stderr, "Failed to create server\n");
        return 4;
    }
    
    if (server_open(hs.skt)<0) {
        fprintf(stderr, "Failed to open server\n");
        return 4;
    }
    fprintf(stderr, "Opened port %d for device %s\n", port, argv[1]);
    if (open_device(hs.dev)<0)  exit(4);
    if (start_device(hs.dev)<0) exit(4);

    signal(SIGINT, sig_handler);
    signal(SIGHUP, sig_handler);
    signal(SIGTERM, sig_handler);

    hs.dev_thread.hs = create_highlight_set(1);
    if (server_start_thread(&hs.srv_thread, hs.skt, 10000, server_data_callback, (void *)&hs)<0){
        fprintf(stderr, "Failed to start server thread\n");
        return 4;
    }
    if (device_start_thread(&hs)<0) {
        fprintf(stderr, "Failed to start device thread\n");
        return 4;
    }
    fprintf(stderr,"Threads spawned\n");
    pthread_join(hs.srv_thread, NULL);
    pthread_join(hs.dev_thread.pthread, NULL);
    fprintf(stderr,"Threads all dead\n");
    server_close(hs.skt);
    server_delete(hs.skt);
    stop_device(hs.dev);
    close_device(hs.dev);
    delete_device(hs.dev);
    return 0;
}
