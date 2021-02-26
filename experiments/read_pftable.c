#include <sys/types.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <sys/fcntl.h>

#include <netinet/in.h>
#include <net/if.h>
#include <net/pfvar.h>
#include <arpa/inet.h>

#include <err.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define SIZE 4

int main() {
    if (getuid() != 0) {
        printf("Must be root\n");
        return 1;
    }

    int dev = open("/dev/pf", O_RDWR);
    printf("Got fd: %u\n", dev);

    struct pfr_table tbl;
    bzero(&tbl, sizeof(tbl));
    strlcpy(tbl.pfrt_name, "my_table", sizeof(tbl.pfrt_name));
    printf("Table Name: %s\n", tbl.pfrt_name);

    int msize = sizeof(struct pfr_addr) * SIZE;
    printf("Malloc %d bytes to store %d pfr_addr structures (%lu each)\n", 
        msize, SIZE, sizeof(struct pfr_addr));
    struct pfr_addr *addr = malloc(msize);
    bzero(addr, msize);

    struct pfioc_table io;
    bzero(&io, sizeof(io));
    io.pfrio_table = tbl;
    io.pfrio_buffer = addr;
    io.pfrio_esize = sizeof(*addr);
    io.pfrio_size = SIZE;

    printf("pfrio_size before ioctl: %d\n", io.pfrio_size);
    if (ioctl(dev, DIOCRGETADDRS, &io) == -1) {
        return -1;
    }
    printf("pfrio_size after ioctl: %d\n", io.pfrio_size);

    if (io.pfrio_size >= 1) {
        int i;
        for (i = 0; i < io.pfrio_size; i++) {
            struct pfr_addr a = addr[i];
            char buffer[INET_ADDRSTRLEN];
            inet_ntop(a.pfra_af, &a.pfra_u, buffer, sizeof(buffer));
            printf("%s\n", buffer);
        }
    }

    free(addr);
    close(dev);
    return 0;
}