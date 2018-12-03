#ifndef __INC_CAPTURE_DEVICE_H
#define __INC_CAPTURE_DEVICE_H
#include <sys/time.h>

enum {
    cd_param_exposure,
    cd_param_brightness,
    cd_param_contrast,
    cd_param_gain,
};
struct capture_device;
struct capture_buffer {
    const unsigned char *buffer;
    int width;
    int height;
    long long timestamp;
};
typedef void t_frame_callback(void *handle, const struct capture_buffer *buffer);
extern t_frame_callback dump_to_file;
extern int capture_frame(struct capture_device *dev, int timeout_us, t_frame_callback *frame_callback, void *handle);
extern int stop_device(struct capture_device *dev);
extern int start_device(struct capture_device *dev);
extern int open_device(struct capture_device *dev);
extern void close_device(struct capture_device *dev);
extern struct capture_device *create_device(const char *device_name);
extern void delete_device(struct capture_device *dev);
extern int cd_set_parameter(struct capture_device *dev, int param, int value);
extern int cd_flush(struct capture_device *dev);
extern long long cd_now_of_local(struct capture_device *dev, struct timeval *local);
extern long long cd_now(void);
#endif
