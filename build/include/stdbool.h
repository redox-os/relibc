#ifndef _STDBOOL_H
#define _STDBOOL_H

#ifndef __cplusplus
typedef _Bool bool;
#define true 1
#define false 0
#else /* __cplusplus */
typedef bool _Bool;
#if __cplusplus < 201103L
#define false false
#define true true
#endif /*__cplusplus < 201103L*/
#endif /* __cplusplus */

#define __bool_true_false_are_defined 1


#endif /* _STDBOOL_H */
