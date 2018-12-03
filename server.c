/*a Copyright */
/*a Doc

convert -depth 8 -size 640x480 a.gray a.jpg

 */
/*a Includes */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <netinet/in.h>
#include <pthread.h>
#include <unistd.h> // for close

#include "server.h"

/*a Types */
/*t server_skt
 */
struct server_skt {
    int skt;
    int thread_started;
    int halt_thread;
    int port;
    int timeout_usec;
    t_server_callback *callback;
    void *callback_handle;
    int tv_sec;
    int tv_usec;
    int client_skt;
    int buffer_valid;
    char buffer[8192];
};

/*a Server
 */

#define STATUS_BASE (20*40)
#define TIMEOUT_SEC  0
#define TIMEOUT_USEC 100000
#define DATA_MODE_COMMAND  0
#define DATA_MODE_RV_DATA  1

/*f server_open */
int server_open(struct server_skt *skt) {
    struct sockaddr_in addr;

    skt->skt = socket(AF_INET, SOCK_STREAM, 0);
    if (skt->skt<0) {return -1;}
    int reuse = 1;
    setsockopt(skt->skt, SOL_SOCKET, SO_REUSEADDR, &reuse, sizeof(reuse));
    bzero((char *) &addr, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(skt->port);
    if (bind(skt->skt, (struct sockaddr *)&addr, sizeof(addr)) < 0) { return -2; }
    if (listen(skt->skt,1)<0) { return -3; }
    return 0;
}

/*f server_close_client */
static
void server_close_client(struct server_skt *skt) {
    if (skt->client_skt>=0) {
        fprintf(stderr, "Closed client\n");
        close(skt->client_skt);
        skt->client_skt = -1;
    }
}

/*f server_close */
int server_close(struct server_skt *skt) {
    server_close_client(skt);
    if (skt->skt>=0) {
        fprintf(stderr, "Closed server\n");
        close(skt->skt);
        skt->skt = -1;
    }
    return 0;
}

/*f server_poll_for_new_client */
static
int server_poll_for_new_client(struct server_skt *skt, int timeout_usec) {
    if (skt->client_skt>0) return 0;
    fd_set read, write, err;
    struct timeval timeout;
    timeout.tv_sec  = 0;
    timeout.tv_usec = timeout_usec;
    FD_ZERO(&read);
    FD_ZERO(&write);
    FD_ZERO(&err);
    FD_SET(skt->skt, &read);
    FD_SET(skt->skt, &err);

    int i = select(skt->skt+1, &read, &write, &err, &timeout);
    if (i<0) return -1;
    if (i==0) return 0;
    if (FD_ISSET(skt->skt, &err)) return -2;
    if (!FD_ISSET(skt->skt, &read)) return 0;

    struct sockaddr_in addr;
    socklen_t addr_len = sizeof(addr);
    fprintf(stderr, "Wait for client\n");
    skt->client_skt = accept(skt->skt, (struct sockaddr *)&addr, &addr_len);
    if (skt->client_skt < 0) { return -3; }
    skt->buffer_valid = 0;
    fprintf(stderr, "Accepted client\n");
    return 0;
}

/*f server_poll_client */
static
int server_poll_client(struct server_skt *skt, int timeout_usec) {
    if (skt->client_skt<=0) return 0;
    fd_set read, write, err;
    struct timeval timeout;
    timeout.tv_sec  = 0;
    timeout.tv_usec = timeout_usec;
    FD_ZERO(&read);
    FD_ZERO(&write);
    FD_ZERO(&err);
    FD_SET(skt->client_skt, &read);
    FD_SET(skt->client_skt, &err);

    int i = select(skt->client_skt+1, &read, &write, &err, &timeout);
    if (i<0) return -1;
    if (i==0) return 0;
    if (FD_ISSET(skt->client_skt, &err)) {
        close(skt->client_skt);
        skt->client_skt = -1;
        return -1;
    }
    if (!FD_ISSET(skt->client_skt, &read)) return 0;

    int n = recv(skt->client_skt, skt->buffer+skt->buffer_valid, sizeof(skt->buffer)-skt->buffer_valid, 0);
    if (n==0) {
        server_close_client(skt);
        return 0;
    }
    skt->buffer_valid += n;
    return skt->buffer_valid;
}

/*f server_add_to_send */
int server_add_to_send(struct server_skt *skt, const char *buffer, int len) {
    if (send(skt->client_skt, buffer, len, 0)<0) return -1;
    return 0;
}

/*f server_poll */
int server_poll(struct server_skt *skt, int timeout_usec, t_server_callback *callback, void *handle) {
    int i = server_poll_for_new_client(skt, timeout_usec);
    if (i<0) return -2;
    while (skt->client_skt>=0) {
        i = callback(skt, handle, skt->buffer, skt->buffer_valid);
        if (i<0) {
            server_close_client(skt);
            return -1;
        }
        if (i==0)
            break;
        memmove(skt->buffer, skt->buffer+i, skt->buffer_valid-i);
        skt->buffer_valid -= i;
    };
    i = server_poll_client(skt, timeout_usec);
    if (i<0) return -1;
    return 0;
}

/*f server_thread
 */
static
void *server_thread(void *handle)
{
    struct server_skt *skt = (struct server_skt *)handle;
    while (!skt->halt_thread) {
        int rc = server_poll(skt, skt->timeout_usec, skt->callback, skt->callback_handle);
        if (rc<-1) break;
    }
    skt->thread_started = 0;
    return 0;
}

/*f server_halt_thread
 */
int server_halt_thread(struct server_skt *skt)
{
    skt->halt_thread = 1;
    return 0;
}

/*f server_start_thread
 */
int server_start_thread(pthread_t *srv_thread, struct server_skt *skt, int timeout_usec, t_server_callback *callback, void *handle)
{
    pthread_attr_t attr;
    int rc=0;
    if (skt->thread_started) {
        fprintf(stderr,"Attempt to start server thread when already running\n");
        return -10;
    }
    skt->callback = callback;
    skt->callback_handle = handle;
    skt->halt_thread = 0;
    skt->timeout_usec = timeout_usec;
    if (pthread_attr_init(&attr)<0) return -1;
    skt->thread_started = 1;
    rc = pthread_create(srv_thread, &attr, server_thread, (void *)skt);
    pthread_attr_destroy(&attr);
    if (rc<0) {skt->thread_started = 0;}
    return rc;
}

/*f server_create
 */
struct server_skt *server_create(int port) {
    struct server_skt *skt = (struct server_skt *)calloc(1, sizeof(*skt));
    skt->port = port;
    skt->callback = NULL;
    skt->callback_handle = NULL;
    skt->halt_thread = 0;
    skt->thread_started = 0;
    return skt;
}

/*f server_delete
 */
void server_delete(struct server_skt *skt) {
    if (skt->thread_started) {
        fprintf(stderr,"Attempt to delete server when thread running\n");
        return;
    }
    if (skt) {
        free(skt);
    }
}
