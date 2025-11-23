// Do not use include guard, to ensure assert is always defined
#ifdef assert
#undef assert
#endif

#ifdef NDEBUG
# define assert(cond) (void) 0
#else
# define assert(cond) \
  ((void)((cond) || (__assert_fail(__func__, __FILE__, __LINE__, #cond), 0)))
#endif
