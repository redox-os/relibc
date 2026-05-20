#include <sys/ipc.h>
#include <sys/shm.h>
#include <string.h>

#include "test_helpers.h"

int main(void)
{
    int status;
    key_t key = ftok("example_dir/1-never-gonna-give-you-up", 123);
    size_t size = 1000;

    int shmid = shmget(key, size, IPC_CREAT | IPC_EXCL | 0666);
    ERROR_IF(shmget, shmid, == -1);

    struct shmid_ds ds;
    status = shmctl(shmid, IPC_STAT, &ds);
    ERROR_IF(shmctl, status, == -1);
    UNEXP_IF(shmctl, ds.shm_segsz, != size);

    void *shmaddr = shmat(shmid, NULL, 0);
    UNEXP_IF(shmat, shmaddr, == SHM_FAILED);

    char *msg = "foo bar";
    strcpy((char *)shmaddr, msg);

    status = strcmp((char *)shmaddr, msg);
    UNEXP_IF(strcmp, status, != 0);

    status = shmdt(shmaddr);
    ERROR_IF(shmdt, status, == -1);

    status = shmctl(shmid, IPC_RMID, NULL);
    ERROR_IF(shmctl, status, == -1);
}