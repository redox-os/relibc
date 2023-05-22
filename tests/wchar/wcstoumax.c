#include <inttypes.h>      
#include <stdio.h>
#include <wchar.h>

int main(void) {                                                    
    wchar_t *nptr;                                    
    wchar_t *endptr;                                  
    uintmax_t  j;                                      
    int base = 10;                                    
    nptr = L"10110134932";                            
    printf("nptr = `%ls`\n", nptr);                   
    j = wcstoumax(nptr, &endptr, base);            
    printf("wcstoumax = %ju\n", j);            
    printf("Stopped scan at `%ls`\n", endptr);
} 