#include <sys/types.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <net/pfvar.h>
#include <stdio.h>

int main() {
    printf("pub const PATH_MAX: u32 = %d;\n", PATH_MAX);
    printf("pub const IFNAMSIZ: u32 = %d;\n", IFNAMSIZ);
    printf("pub const INET_ADDRSTRLEN: u32 = %u;\n", INET_ADDRSTRLEN);
    printf("pub const PF_TABLE_NAME_SIZE: u32 = %d;\n", PF_TABLE_NAME_SIZE);
    printf("pub const DIOCRSETADDRS: u64 = %lu;\n", DIOCRSETADDRS);
    printf("pub const DIOCRGETADDRS: u64 = %lu;\n", DIOCRGETADDRS);
    return 0;
}