#include "capture_device.h"

struct highlight_set;
extern void highlight_set_precapture(struct highlight_set *hs, int n);
extern void highlight_set_complete(struct highlight_set *hs);
extern struct highlight_set *create_highlight_set(int num_devices);
extern void find_highlights(void *handle, const struct capture_buffer *cb);
