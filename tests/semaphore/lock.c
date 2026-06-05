#include <semaphore.h>
#include <errno.h>

#include "test_helpers.h"

int main(void)
{
    sem_t sem;
    int status;

    status = sem_init(&sem, 0, 1);
    ERROR_IF(sem_init, status, == -1);

    status = sem_trywait(&sem);
    ERROR_IF(sem_trywait, status, == -1);

    status = sem_trywait(&sem);
    UNEXP_IF(sem_trywait, status, != -1);
    UNEXP_IF(errno, errno, != EAGAIN);

    status = sem_post(&sem);
    ERROR_IF(sem_post, status, == -1);

    status = sem_wait(&sem);
    ERROR_IF(sem_wait, status, == -1);

    status = sem_destroy(&sem);
    ERROR_IF(sem_destroy, status, == -1);
}