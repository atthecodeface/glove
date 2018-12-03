/*a Copyright */
/*a Doc

convert -depth 8 -size 640x480 a.gray a.jpg

 */
/*a Includes */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "capture_device.h"
#include "highlights.h"

/*a Types */
/*a Functions */
/*f Main */
int main(int argc, char **argv)
{
    struct capture_device *dev[8];
    int i, num_devices;
    if (argc<2) {
        fprintf(stderr, "Usage %s <device> [<devices> ...]\n", argv[0]);
        return 4;
    }
    num_devices=0;
    for (i=1; i<argc; i++) {
        dev[num_devices++] = create_device(argv[i]);
    }
    struct highlight_set *hs = create_highlight_set(num_devices);
    for (i=0; i<num_devices; i++) {
        if (open_device(dev[i])<0)  exit(4);
    }
    for (i=0; i<num_devices; i++) {
        if (start_device(dev[i])<0) exit(4);
    }
    for (i=0; i<2000; i++) {
        int j;
        for (j=0; j<num_devices; j++) {
            highlight_set_precapture(hs, j);
            capture_frame(dev[j], 1000*1000*4, find_highlights, hs);
        }
        highlight_set_complete(hs);
    }
    for (i=0; i<num_devices; i++) {
        stop_device(dev[i]);
        close_device(dev[i]);
        delete_device(dev[i]);
    }
    return 0;
}
