go: capture
	./capture /dev/video0
	convert -depth 8 -size 640x480 a.gray a.jpg
	./capture /dev/video1
	convert -depth 8 -size 640x480 a.gray b.jpg
	./capture /dev/video2
	convert -depth 8 -size 640x480 a.gray c.jpg

.PHONY:track_glove
track_glove:
	jbuilder build src/track_glove/track_glove.exe
	#DYLD_LIBRARY_PATH=~/Git/v4l/_build/default/src/sdl_ogl_gui/sdl
	_build/default/src/track_glove/track_glove.exe

track: track_highlights
	./track_highlights /dev/video0 /dev/video1 /dev/video2

all: capture track_highlights highlights_server

capture: capture.o capture_device.o highlights.o
	gcc -o capture capture.o capture_device.o highlights.o

track_highlights: track_highlights.o capture_device.o highlights.o
	gcc -o track_highlights track_highlights.o capture_device.o highlights.o

highlights_server: highlights.o capture_device.o highlights_server.o server.o
	gcc -o $@ highlights.o capture_device.o highlights_server.o server.o -lpthread

hs3: highlights_server
	((echo "shutdown\n" | netcat 127.0.0.1 1234) || true)
	((echo "shutdown\n" | netcat 127.0.0.1 1235) || true)
	((echo "shutdown\n" | netcat 127.0.0.1 1236) || true)
	./highlights_server /dev/video0 & ./highlights_server /dev/video1 & ./highlights_server /dev/video2 &

capture_hs3:
	( ((echo "set 1 128\nset 2 32\nset 3 64\ndump\nclose\n" | netcat 127.0.0.1 1234) || true) & \
	  ((echo "set 1 128\nset 2 32\nset 3 64\ndump\nclose\n" | netcat 127.0.0.1 1235) || true) & \
	  ((echo "set 1 128\nset 2 32\nset 3 64\ndump\nclose\n" | netcat 127.0.0.1 1236) || true) )
	sleep 1
	convert -depth 8 -size 640x480 a0.gray a0.jpg
	convert -depth 8 -size 640x480 a1.gray a1.jpg
	convert -depth 8 -size 640x480 a2.gray a2.jpg
	mirage a0.jpg

kill_hs3: highlights_server
	((echo "shutdown\n" | netcat 127.0.0.1 1234) || true)
	((echo "shutdown\n" | netcat 127.0.0.1 1235) || true)
	((echo "shutdown\n" | netcat 127.0.0.1 1236) || true)


%.o:%.c capture_device.h highlights.h server.h
	gcc -c $< -o $@

#                     brightness 0x00980900 (int)    : min=0 max=255 step=1 default=128 value=128
#                       contrast 0x00980901 (int)    : min=0 max=255 step=1 default=32 value=32
#                     saturation 0x00980902 (int)    : min=0 max=255 step=1 default=32 value=32
# white_balance_temperature_auto 0x0098090c (bool)   : default=1 value=1
#                           gain 0x00980913 (int)    : min=0 max=255 step=1 default=64 value=192
#           power_line_frequency 0x00980918 (menu)   : min=0 max=2 default=2 value=2
#      white_balance_temperature 0x0098091a (int)    : min=0 max=10000 step=10 default=4000 value=1690 flags=inactive
#                      sharpness 0x0098091b (int)    : min=0 max=255 step=1 default=24 value=24
#         backlight_compensation 0x0098091c (int)    : min=0 max=1 step=1 default=0 value=0
#                  exposure_auto 0x009a0901 (menu)   : min=0 max=3 default=3 value=3 (1 is manual I believe)
#              exposure_absolute 0x009a0902 (int)    : min=1 max=10000 step=1 default=166 value=254 flags=inactive
#         exposure_auto_priority 0x009a0903 (bool)   : default=0 value=1

# Cannot do this:
CONTROLS = --set-ctrl=power_line_frequency=0 --set-ctrl=saturation=0 --set-ctrl=white_balance_temperature_auto=0 --set-ctrl=white_balance_temperature=2000 --set-ctrl=exposure_auto=1 --set-ctrl=exposure_absolute=40 --set-ctrl=brightness=80 --set-ctrl=contrast=64 --set-ctrl=gain=15

video_controls:
	v4l2-ctl -d /dev/video0 $(CONTROLS)
	v4l2-ctl -d /dev/video1 $(CONTROLS)
	v4l2-ctl -d /dev/video0 -L
	v4l2-ctl -d /dev/video1 -L
