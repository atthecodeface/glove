/*a Copyright */
/*a Doc

convert -depth 8 -size 640x480 a.gray a.jpg

 */
/*a Includes */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "capture_device.h"

/*a Types */
/*a Useful functions */
/*f Main for now */
int main(int argc, char **argv)
{
    struct capture_device *dev;
    int i;
    if (argc<2) {
        fprintf(stderr, "Usage %s <device>\n", argv[0]);
        return 4;
    }
    dev = create_device(argv[1]);
	if (open_device(dev)<0)  exit(4);
	if (start_device(dev)<0) exit(4);
    capture_frame(dev, 1000*1000*4, dump_to_file, (void *)"a.gray");
	stop_device(dev);
	close_device(dev);
    delete_device(dev);
    return 0;
}
