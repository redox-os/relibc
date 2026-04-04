//Take from https://www.ibm.com/docs/en/zos/3.1.0?topic=functions-fmtmsg-display-message-in-specified-format
#include <fmtmsg.h>
int main() {
    fmtmsg(MM_PRINT, "XSI:cat", MM_ERROR, "illegal option",
    "refer to cat in user's reference manual", "XSI:cat:001");
    //expected output: 
    //XSI:cat: ERROR: illegal option
    //TO FIX: refer to cat in user's reference manual XSI:cat:001
    return 0;
    
}