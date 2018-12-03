/*a Copyright */
/*a Doc

convert -depth 8 -size 640x480 a.gray a.jpg

 */
/*a Includes */
#include "capture_device.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

#include <getopt.h>             /* getopt_long() */

#include <fcntl.h>              /* low-level i/o */
#include <unistd.h>
#include <errno.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/time.h>
#include <time.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <linux/videodev2.h>

/*a Types */
#define MAX_BUFFERS 4
struct capture_device {
    const char *name;
    int fd;
    unsigned int buffer_size;
    unsigned int num_buffers;
    long long clock_delta_us;
    void *buffers[MAX_BUFFERS];
};

/*a Standard frame callbacks */
/*f dump_to_file - capture frame callback, dumps to a.yuv */
void dump_to_file(void *handle, const struct capture_buffer *cb)
{
    FILE *f = fopen((char *)handle,"w");
    int i;
    for (i=0; i<cb->width*cb->height*2; i+=2) {
        fwrite(cb->buffer+i, 1, 1, f);
    }
    fclose(f);
}

/*a Functions */
/*f cd_now */
long long cd_now(void) {
    struct timeval now;
    gettimeofday(&now, NULL);
    return (now.tv_sec*1000LL*1000LL +
            now.tv_usec);
}

/*f cd_now_of_local */
long long cd_now_of_local(struct capture_device *dev, struct timeval *local) {
    return dev->clock_delta_us + (local->tv_sec*1000LL*1000LL +
                                  local->tv_usec);
}

/*f try_to_set */
static int try_to_set(struct capture_device *dev, int id, int value)
{
    struct v4l2_control c;
    c.id = id;
    c.value = value;
    return ioctl(dev->fd, VIDIOC_S_CTRL, &c);
}
/*f cd_set_parameter */
int cd_set_parameter(struct capture_device *dev, int param, int value)
{
    switch (param) {
    case cd_param_exposure:
        return try_to_set(dev, V4L2_CID_EXPOSURE, value);
    case cd_param_brightness:
        return try_to_set(dev, V4L2_CID_BRIGHTNESS, value);
    case cd_param_contrast:
        return try_to_set(dev, V4L2_CID_CONTRAST, value);
    case cd_param_gain:
        return try_to_set(dev, V4L2_CID_GAIN, value);
    }
    return -1;
}

/*f cd_poll */
int cd_poll(struct capture_device *dev, int timeout_us)
{
    fd_set fds;
    struct timeval tv;

    /*b Poll */
    FD_ZERO(&fds);
    FD_SET(dev->fd, &fds);
    tv.tv_sec = 0;
    tv.tv_usec = timeout_us;
    return select(dev->fd + 1, &fds, NULL, NULL, &tv);
}

/*f capture_frame */
int capture_frame(struct capture_device *dev,
                  int timeout_us,
                  t_frame_callback *frame_callback,
                  void *handle )
{
    int rc;
    struct v4l2_buffer buf;

    /*b Poll */
    rc = cd_poll(dev, timeout_us);
    if (rc==0) return 0; // timeout
    if (rc<0)  return -1; // error

    /*b Dequeue a buffer */
    buf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    buf.memory = V4L2_MEMORY_MMAP;
    buf.index = 0;
    if (ioctl(dev->fd, VIDIOC_DQBUF, &buf)<0) {
        fprintf(stderr,"Failed to dequeue buffer\n");
        return -1;
    }
    if (buf.index>=dev->num_buffers) return -1;

    /*b Invoke callback */
    if (frame_callback) {
        struct capture_buffer cb;
        cb.buffer = dev->buffers[buf.index];
        cb.width = 640;
        cb.height = 480;
        cb.timestamp = cd_now_of_local(dev, &buf.timestamp);
        frame_callback(handle, &cb );
    }

    /*b Enqueue buffer again */
    if (ioctl(dev->fd, VIDIOC_QBUF, &buf)<0) return -1;
    return 1;
}

/*f cd_flush */
int cd_flush(struct capture_device *dev)
{
    struct v4l2_buffer buf;
    int n=0;
    while (cd_poll(dev, 4000)>0) {
        buf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory = V4L2_MEMORY_MMAP;
        buf.index = 0;
        if (ioctl(dev->fd, VIDIOC_DQBUF, &buf)>=0) {
            ioctl(dev->fd, VIDIOC_QBUF, &buf);
            n++;
        }
    }
    return n;
}

/*f stop_device */
int stop_device(struct capture_device *dev)
{
    enum v4l2_buf_type type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    if (dev->fd<0) return 0;
    return ioctl(dev->fd, VIDIOC_STREAMOFF, &type);
}

/*f start_device */
int start_device(struct capture_device *dev)
{
    unsigned int i;
    enum v4l2_buf_type type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    struct v4l2_buffer buf;

    if (dev->fd<0) {
        fprintf(stderr,"Device not ready to start\n");
        return -1;
    }

    memset(&buf, 0, sizeof(buf));
    buf.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    buf.memory = V4L2_MEMORY_MMAP;
    for (i=0; i<dev->num_buffers; i++) {
        buf.index = i;
        if (ioctl(dev->fd, VIDIOC_QBUF, &buf)<0) {
            fprintf(stderr,"Failed to enqueue buffer %d\n", i);
            return -1;
        }
    }
    if (ioctl(dev->fd, VIDIOC_STREAMON, &type)<0) {
        fprintf(stderr,"Failed to start video streaming\n");
        return -1;
    }
    return 0;
}

/*f close_device */
void close_device(struct capture_device *dev)
{
    unsigned int i;

    if (dev->fd<0) return;
    for (i=0; i<dev->num_buffers; i++) {
        if (dev->buffers[i]) {
            munmap(dev->buffers[i], dev->buffer_size);
            dev->buffers[i] = NULL;
        }
    }
    close(dev->fd);
    dev->fd = -1;
}

/*f open_device */
int open_device(struct capture_device *dev)
{
    struct v4l2_capability cap;
    struct v4l2_format fmt;
    struct v4l2_requestbuffers req;
    int i;

    dev->fd = open(dev->name, O_RDWR | O_NONBLOCK, 0);
    if (dev->fd<0) {
        fprintf(stderr, "Failed to open device '%s'\n", dev->name);
        return -1;
    }
    if (ioctl(dev->fd, VIDIOC_QUERYCAP, &cap)<0) {
        fprintf(stderr, "%s does not support V4L2\n", dev->name);
        return -1;
    }

    memset(&fmt, 0, sizeof(fmt));

    fmt.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    fmt.fmt.pix.width       = 640;
    fmt.fmt.pix.height      = 480;
    fmt.fmt.pix.pixelformat = V4L2_PIX_FMT_YUYV;
    //fmt.fmt.pix.field       = V4L2_FIELD_INTERLACED;

    if (ioctl(dev->fd, VIDIOC_S_FMT, &fmt)<0) {
        fprintf(stderr, "%s does not support YUYV capture format\2n", dev->name);
        return -1;
    }
    try_to_set(dev, V4L2_CID_3A_LOCK, V4L2_LOCK_FOCUS | V4L2_LOCK_WHITE_BALANCE | V4L2_LOCK_EXPOSURE );
    try_to_set(dev, V4L2_CID_EXPOSURE_AUTO, V4L2_EXPOSURE_MANUAL);
    try_to_set(dev, V4L2_CID_AUTO_FOCUS_RANGE, V4L2_AUTO_FOCUS_RANGE_MACRO);
    try_to_set(dev, V4L2_CID_ISO_SENSITIVITY_AUTO, V4L2_ISO_SENSITIVITY_MANUAL);
    try_to_set(dev, V4L2_CID_EXPOSURE, 40);
    try_to_set(dev, V4L2_CID_BRIGHTNESS, 65);
    try_to_set(dev, V4L2_CID_CONTRAST, 64);
    try_to_set(dev, V4L2_CID_GAIN, 15);
    try_to_set(dev, V4L2_CID_HUE, 0);
    try_to_set(dev, V4L2_CID_SATURATION, 0);
    try_to_set(dev, V4L2_CID_POWER_LINE_FREQUENCY, V4L2_CID_POWER_LINE_FREQUENCY_DISABLED);
    try_to_set(dev, V4L2_CID_WHITE_BALANCE_TEMPERATURE, 1600);
    try_to_set(dev, V4L2_CID_SHARPNESS, 24);
    try_to_set(dev, V4L2_CID_AUTOBRIGHTNESS, 0);
    // V4L2_CID_AUTO_WHITE_BALANCE    (V4L2_CID_BASE+12)
    // V4L2_CID_GAMMA            (V4L2_CID_BASE+16)
    // V4L2_CID_AUTOGAIN        (V4L2_CID_BASE+18)
    // V4L2_CID_HFLIP            (V4L2_CID_BASE+20)
    // V4L2_CID_VFLIP            (V4L2_CID_BASE+21)
    // V4L2_CID_ROTATE                (V4L2_CID_BASE+34)
    // V4L2_CID_BG_COLOR            (V4L2_CID_BASE+35)


    memset(&req, 0, sizeof(req));
    req.count = 4;
    req.type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
    req.memory = V4L2_MEMORY_MMAP;

    if (ioctl(dev->fd, VIDIOC_REQBUFS, &req)<0) {
        fprintf(stderr, "Failed to request %d buffers %s\n", req.count, dev->name);
        return -1;
    }

    if (req.count < 2) {
        fprintf(stderr, "Insufficient buffer memory on %s\n", dev->name);
        return -1;
    }

    for (i=0; i<req.count; i++) {
        struct v4l2_buffer buf;

        memset(&buf, 0, sizeof(buf));

        buf.type        = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        buf.memory      = V4L2_MEMORY_MMAP;
        buf.index       = i;

        if (ioctl(dev->fd, VIDIOC_QUERYBUF, &buf)) {
            fprintf(stderr, "Could not query buffer %d on %s\n", i, dev->name);
            return -1;
        }

        dev->buffer_size = buf.length;
        dev->buffers[i] = mmap(NULL, buf.length, PROT_READ | PROT_WRITE, MAP_SHARED, dev->fd, buf.m.offset);

        if (dev->buffers[i]==MAP_FAILED) {
            fprintf(stderr, "Could not map buffer %d on %s\n", i, dev->name);
            return -1;
        }
    }
    dev->num_buffers = req.count;
    return 0;
}

/*f create_device */
struct capture_device *create_device(const char *device_name)
{
    struct capture_device *dev;
    dev = (struct capture_device *)malloc(sizeof(struct capture_device));
    dev->name = (const char *)malloc(strlen(device_name)+1);
    strcpy((char *)dev->name, device_name);

    struct timeval now;
    struct timespec uptime;
    gettimeofday(&now, NULL);
    clock_gettime(CLOCK_MONOTONIC, &uptime);
    dev->clock_delta_us = ( (now.tv_sec-uptime.tv_sec)*1000LL*1000LL +
                            (now.tv_usec-uptime.tv_nsec/1000LL) );
    return dev;
}

/*f delete_device */
void delete_device(struct capture_device *dev)
{
    if (!dev) return;
    stop_device(dev);
    close_device(dev);
    free((void *)(dev->name));
    free(dev);
}
