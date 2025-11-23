// Copyright (C) Dahua Lin, 2014. Provided under the MIT license.

// Benchmark on libm functions

#include <math.h>
#include <time.h>
#include <stdio.h>
#include <stdlib.h>


// Timing facilities

#ifdef __MACH__

#include <mach/mach_time.h>

class stimer
{
public:
    typedef uint64_t time_type;

    stimer()
    {
        ::mach_timebase_info(&m_baseinfo);
    }

    time_type current() const
    {
        return ::mach_absolute_time();
    }

    double span(const time_type& t0, const time_type& t1) const
    {
        uint64_t d = (m_baseinfo.numer * (t1 - t0)) / m_baseinfo.denom;
        return static_cast<double>(d) / 1.0e9;
    }

private:
    mach_timebase_info_data_t m_baseinfo;
};

#else

class stimer
{
public:
    typedef timespec time_type;

    time_type current() const
    {
        time_type t;
        ::clock_gettime(CLOCK_REALTIME, &t);
        return t;
    }

    double span(const time_type& t0, const time_type& t1) const
    {
        return double(t1.tv_sec - t0.tv_sec) +
            double(t1.tv_nsec - t0.tv_nsec) * 1.0e-9;
    }
};

#endif


inline double sec2mps(double s, long n)
{
    return n / (s * 1e6);
}


const long ARR_LEN = 1024;

double a[ARR_LEN];
double b[ARR_LEN];
double r[ARR_LEN];

#define TFUN1(FNAME) \
    void test_##FNAME(long n) { \
        for (int j = 0; j < ARR_LEN; ++j) r[j] = FNAME(a[j]); \
        stimer tm; \
        stimer::time_type t0 = tm.current(); \
        for(int i = 0; i < n; ++i) { \
            for (int j = 0; j < ARR_LEN; ++j) r[j] = FNAME(a[j]); \
        } \
        double s = tm.span(t0, tm.current()); \
        double mps = sec2mps(s, n * ARR_LEN); \
        printf("  %-8s:  %7.4f MPS\n", #FNAME, mps); }

#define TFUN2(FNAME) \
    void test_##FNAME(long n) { \
        for (int j = 0; j < ARR_LEN; ++j) r[j] = FNAME(a[j], b[j]); \
        stimer tm; \
        stimer::time_type t0 = tm.current(); \
        for(int i = 0; i < n; ++i) { \
            for (int j = 0; j < ARR_LEN; ++j) r[j] = FNAME(a[j], b[j]); \
        } \
        double s = tm.span(t0, tm.current()); \
        double mps = sec2mps(s, n * ARR_LEN); \
        printf("  %-8s:  %7.4f MPS\n", #FNAME, mps); }


#define TCALL(FNAME) test_##FNAME(20000)

// define benchmark functions

TFUN2(pow)
TFUN2(hypot)

TFUN1(exp)
TFUN1(log)
TFUN1(log10)
TFUN1(sin)
TFUN1(cos)
TFUN1(tan)
TFUN1(asin)
TFUN1(acos)
TFUN1(atan)
TFUN2(atan2)

int main(int argc, char *argv[])
{
    // initialize array contents
    for (int i = 0; i < ARR_LEN; ++i)
    {
      a[i] = rand() / (double) RAND_MAX;
      b[i] = rand() / (double) RAND_MAX;
    }

    TCALL(pow);
    TCALL(hypot);
    TCALL(exp);
    TCALL(log);
    TCALL(log10);
    TCALL(sin);
    TCALL(cos);
    TCALL(tan);
    TCALL(asin);
    TCALL(acos);
    TCALL(atan);
    TCALL(atan2);

    return 0;
}
