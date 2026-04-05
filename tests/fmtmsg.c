#include <fmtmsg.h>
#include <unistd.h>
int main() {
    /* Returned Value
     * MM_OK: 0 - The function succeeded.
     * MM_NOCON: 4 - The function was unable to generate a console message, but otherwise succeeded.
     * MM_NOMSG: 1 - The function was unable to generate a message on standard error, but otherwise succeeded.
     * MM_NOTOK: -1 - The function failed completely.
     * 
     * Take from: https://www.ibm.com/docs/en/zos/3.1.0?topic=functions-fmtmsg-display-message-in-specified-format
     */
    int mm_ok = fmtmsg(MM_PRINT, "XSI:cat", MM_ERROR, "illegal option",
    "refer to cat in user's reference manual", "XSI:cat:001");
    if (mm_ok != 0) {
        return -1;
    }
    int mm_nocon = fmtmsg(MM_PRINT | MM_CONSOLE, "XSI:cat", MM_ERROR, "illegal option",
    "refer to cat in user's reference manual", "XSI:cat:001");
    if (mm_nocon != MM_NOCON) {
        return -1;
    }
    //We close stderr to test.
    close(STDERR_FILENO);
    int mm_nomsg = fmtmsg(MM_PRINT , "XSI:cat", MM_ERROR, "illegal option",
    "refer to cat in user's reference manual", "XSI:cat:001");
    if (mm_nomsg != MM_NOMSG) {
        return -1;
    }
    // Thats return 1 in glibc but return -1 in musl
    // Soo we expect -1 here
    int mm_notok = fmtmsg(MM_PRINT | MM_CONSOLE, "XSI:cat", MM_ERROR, "illegal option",
    "refer to cat in user's reference manual", "XSI:cat:001");
    if (mm_notok != MM_NOTOK) {
        return -1;
    }
    return 0;    
}