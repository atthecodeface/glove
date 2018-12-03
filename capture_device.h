enum {
    cd_param_exposure,
    cd_param_brightness,
    cd_param_contrast,
    cd_param_gain,
};
struct capture_device;
typedef void t_frame_callback(void *handle, const unsigned char *buffer, int width, int height);
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
