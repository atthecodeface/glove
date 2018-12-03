struct server_skt;
typedef int t_server_callback(struct server_skt *skt, void *handle, char *in_buffer, int in_valid);
extern struct server_skt *server_create(int port);
extern void server_delete(struct server_skt *skt);
extern int server_open(struct server_skt *skt);
extern int server_close(struct server_skt *skt);
extern int server_poll(struct server_skt *skt, int timeout_usec, t_server_callback *callback, void *handle);
extern int server_add_to_send(struct server_skt *skt, const char *buffer, int len);
extern int server_start_thread(pthread_t *srv_thread, struct server_skt *skt, int timeout_usec, t_server_callback *callback, void *handle);
extern int server_halt_thread(struct server_skt *skt);

