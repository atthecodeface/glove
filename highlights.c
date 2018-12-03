/*a Copyright */
/*a Doc

convert -depth 8 -size 640x480 a.gray a.jpg

 */
/*a Includes */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "highlights.h"

/*f find_highlights */
static int thresh0 = 200;
static int thresh1 = 150;
#define STEP 1
#define MAX_ACTIVE_HIGHTLIGHTS 4
struct highlight {
    int last_y;
    int lx;
    int rx;
    int total_points;
    int dbl_sum_x;
    int sum_y;
};

typedef void t_highlight_find_callback(struct highlight_set *, struct highlight *highlight);
struct highlight_set {
    t_highlight_find_callback *find_callback;
    struct highlight highlights[MAX_ACTIVE_HIGHTLIGHTS];
    int current_device;
    int num_devices;
};

/*f display_highlight */
static void
display_highlight(struct highlight_set *hs, struct highlight *h)
{
    fprintf(stdout,"(%d,%d,%d,%d);",
            hs->current_device,
            h->total_points,
            h->dbl_sum_x,
            h->sum_y );
}

/*f create_highlight_set */
struct highlight_set *create_highlight_set(int num_devices) {
    struct highlight_set *hs;
    hs = (struct highlight_set *)malloc(sizeof(*hs));
    hs->find_callback = display_highlight;
    hs->num_devices = num_devices;
    return hs;
}

/*f highlight_set_precapture */
void highlight_set_precapture(struct highlight_set *hs, int n) {
    hs->current_device=n;
    if (n==0) {
        fprintf(stdout,"[");
    }
}

/*f highlight_set_complete */
void highlight_set_complete(struct highlight_set *hs) {
    fprintf(stdout,"];\n");
}

/*f init_highlights */
static
void init_highlights(struct highlight_set *hs) {
    int i;
    for (i=0; i<MAX_ACTIVE_HIGHTLIGHTS; i++) {
        hs->highlights[i].last_y = -1;
    }
}

/*f add_highlight */
static
void add_highlight(struct highlight_set *hs, int lx, int rx, int y ) {
    int i;
    int spare=-1, added=0;
    //fprintf(stderr,"%3d:(%3d,%3d)\n",y,lx,rx);
    for (i=0; i<MAX_ACTIVE_HIGHTLIGHTS; i++) {
        if (hs->highlights[i].last_y<0) {spare=i;}
        else {
            if ((lx<hs->highlights[i].rx) && (hs->highlights[i].lx<rx)) {
                hs->highlights[i].total_points += rx-lx;
                hs->highlights[i].lx = lx;
                hs->highlights[i].rx = rx;
                hs->highlights[i].dbl_sum_x += (rx-lx)*(rx+lx);
                hs->highlights[i].sum_y += (rx-lx)*y;
                hs->highlights[i].last_y = y;
                added = 1;
            }
        }
    }
    if (!added && (spare>=0)) {
        hs->highlights[spare].total_points = rx-lx;
        hs->highlights[spare].lx = lx;
        hs->highlights[spare].rx = rx;
        hs->highlights[spare].dbl_sum_x = (rx-lx)*(rx+lx);
        hs->highlights[spare].sum_y = (rx-lx)*y;
        hs->highlights[spare].last_y = y;
    }
}

/*f find_highlights */
void find_highlights(void *handle, const unsigned char *buffer, int width, int height )
{
    struct highlight_set *hs = (struct highlight_set *)handle;
    int i;
    int x, y;
    int num_highlights=0;
    init_highlights(hs);
    for (y=0; y<height; y+=STEP) {
        int lx, rx, state;
        state = 0;
        const unsigned char *ptr = buffer + y*width*2;
        for (x=0; x<width; x+=STEP) {
            int ch=ptr[x*2];
            if (ch<thresh1) { // black
                if (state==2) { add_highlight(hs, lx, x, y); }
                if (state==3) { add_highlight(hs, lx, rx, y); }
                state = 0;
            } else if (ch<thresh0) { // middling
                if (state==3) { rx = x; state=2; }
                else if (state==0) { state=1; }
            } else { // white
                if (state==0) { lx=x; }
                if (state==1) { lx=x; }
                state = 2;
            }
        }
        for (i=0; i<MAX_ACTIVE_HIGHTLIGHTS; i++) {
            if (hs->highlights[i].last_y!=y) {
                if (hs->highlights[i].last_y>0) {
                    hs->find_callback(hs, hs->highlights+i);
                    hs->highlights[i].last_y=-1;
                    num_highlights++;
                }
            }
        }
    }
}

