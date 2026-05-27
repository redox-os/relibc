/*[OB]*/
/* Test whether a basic inet_addr invocation works. */

#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>


int main(void)
{
	in_addr_t value = 0;
	const char* str;

	value = inet_addr(str = "1.2.3.4");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "0XFE.0xdb.1234");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "0x0.007777");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "0xabcdef01");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = " 1.2.3.4");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "0654.2.3.4");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "1.2.3.0x100");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "1.0x1abcdef");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	value = inet_addr(str = "1.2.3.4.5");
	printf("inet_addr(\"%s\") = 0x%08x\n", str, value);

	return 0;
}
