#ifndef _STDBOOL_H
#define _STDBOOL_H

#ifndef __cplusplus
#define bool _Bool
#define true 1
#define false 0
#else /* __cplusplus */
#if __cplusplus < 201103L
#define bool bool
#define false false
#define true true
#endif /*__cplusplus < 201103L*/
#endif /* __cplusplus */

#define __bool_true_false_are_defined 1

#endif /* _STDBOOL_H */
