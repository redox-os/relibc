#ifndef _BITS_SYS_TIME
#define _BITS_SYS_TIME

#define timeradd(x,y,res) (void) (\
        (res)->tv_sec = (x)->tv_sec + (y)->tv_sec + (((x)->tv_usec + (y)->tv_usec) / 1000000), \
        (res)->tv_usec = ((x)->tv_usec + (y)->tv_usec) % 1000000 \
    )
#define timersub(x,y,res) (void) ( \
        (res)->tv_sec = (x)->tv_sec - (y)->tv_sec, \
        (res)->tv_usec = ((x)->tv_usec - (y)->tv_usec), \
        ((res)->tv_usec < 0) && ((res)->tv_sec -= 1, (res)->tv_usec += 1000000) \
    )
#define timerclear(t) (void) ( \
        (t)->tv_sec = 0, \
        (t)->tv_usec = 0 \
    )
#define timerisset(t) ((t)->tv_sec || (t)->tv_usec)
#define timercmp(x,y,op) ((x)->tv_sec == (y)->tv_sec ? \
    (x)->tv_usec op (y)->tv_usec \
    : \
    (x)->tv_sec op (y)->tv_sec)

#endif
