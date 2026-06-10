#include <assert.h>
#include <arpa/inet.h>


int main(void)
{
	unsigned char ip[4];
	const char* input;

	assert(inet_pton(AF_INET, input = "11.22.33.44", ip) == 1);
	assert(ip[0] == 11);
	assert(ip[1] == 22);
	assert(ip[2] == 33);
	assert(ip[3] == 44);

	assert(inet_pton(AF_INET, input = "0.0.0.0", ip) == 1);
	assert(ip[0] == 0);
	assert(ip[1] == 0);
	assert(ip[2] == 0);
	assert(ip[3] == 0);

	assert(inet_pton(AF_INET, input = "255.255.255.255", ip) == 1);
	assert(ip[0] == 255);
	assert(ip[1] == 255);
	assert(ip[2] == 255);
	assert(ip[3] == 255);

	assert(inet_pton(AF_INET, input = "0000.0.0.0", ip) == 0);

	assert(inet_pton(AF_INET, input = "0001.2.3.4", ip) == 0);

	assert(inet_pton(AF_INET, input = "256.256.256.256", ip) == 0);

	assert(inet_pton(AF_INET, input = "1.2.3.0x4", ip) == 0);

	assert(inet_pton(AF_INET, input = "1.2.3", ip) == 0);

	assert(inet_pton(AF_INET, input = "1.2", ip) == 0);

	assert(inet_pton(AF_INET, input = "123456789", ip) == 0);

	assert(inet_pton(AF_INET, input = " 1.2.3.4", ip) == 0);

	assert(inet_pton(AF_INET, input = "1.2.3.4 ", ip) == 0);

#ifdef AF_INET6
	unsigned char ip6[16];

	input = "1122:3344:5566:7788:99aa:bbcc:ddee:ff00";
	assert(inet_pton(AF_INET6, input, ip6) == 1);
	assert(ip6[0] == 0x11);
	assert(ip6[1] == 0x22);
	assert(ip6[2] == 0x33);
	assert(ip6[3] == 0x44);
	assert(ip6[4] == 0x55);
	assert(ip6[5] == 0x66);
	assert(ip6[6] == 0x77);
	assert(ip6[7] == 0x88);
	assert(ip6[8] == 0x99);
	assert(ip6[9] == 0xaa);
	assert(ip6[10] == 0xbb);
	assert(ip6[11] == 0xcc);
	assert(ip6[12] == 0xdd);
	assert(ip6[13] == 0xee);
	assert(ip6[14] == 0xff);
	assert(ip6[15] == 0x00);

	// Test an address with :: notation in the middle.
	input = "1:2::ff";
	assert(inet_pton(AF_INET6, input, ip6) == 1);
	assert(ip6[0] == 0x00);
	assert(ip6[1] == 0x01);
	assert(ip6[2] == 0x00);
	assert(ip6[3] == 0x02);
	assert(ip6[4] == 0x00);
	assert(ip6[5] == 0x00);
	assert(ip6[6] == 0x00);
	assert(ip6[7] == 0x00);
	assert(ip6[8] == 0x00);
	assert(ip6[9] == 0x00);
	assert(ip6[10] == 0x00);
	assert(ip6[11] == 0x00);
	assert(ip6[12] == 0x00);
	assert(ip6[13] == 0x00);
	assert(ip6[14] == 0x00);
	assert(ip6[15] == 0xff);

	input = "::12:0:abcd";
	assert(inet_pton(AF_INET6, input, ip6) == 1);
	assert(ip6[0] == 0x00);
	assert(ip6[1] == 0x00);
	assert(ip6[2] == 0x00);
	assert(ip6[3] == 0x00);
	assert(ip6[4] == 0x00);
	assert(ip6[5] == 0x00);
	assert(ip6[6] == 0x00);
	assert(ip6[7] == 0x00);
	assert(ip6[8] == 0x00);
	assert(ip6[9] == 0x00);
	assert(ip6[10] == 0x00);
	assert(ip6[11] == 0x12);
	assert(ip6[12] == 0x00);
	assert(ip6[13] == 0x00);
	assert(ip6[14] == 0xab);
	assert(ip6[15] == 0xcd);

	// Test an address with omitted zeros and :: notation.
	input = "abcd:0:1::";
	assert(inet_pton(AF_INET6, input, ip6) == 1);
	assert(ip6[0] == 0xab);
	assert(ip6[1] == 0xcd);
	assert(ip6[2] == 0x00);
	assert(ip6[3] == 0x00);
	assert(ip6[4] == 0x00);
	assert(ip6[5] == 0x01);
	assert(ip6[6] == 0x00);
	assert(ip6[7] == 0x00);
	assert(ip6[8] == 0x00);
	assert(ip6[9] == 0x00);
	assert(ip6[10] == 0x00);
	assert(ip6[11] == 0x00);
	assert(ip6[12] == 0x00);
	assert(ip6[13] == 0x00);
	assert(ip6[14] == 0x00);
	assert(ip6[15] == 0x00);

	input = "::";
	assert(inet_pton(AF_INET6, input, ip6) == 1);
	assert(ip6[0] == 0x00);
	assert(ip6[1] == 0x00);
	assert(ip6[2] == 0x00);
	assert(ip6[3] == 0x00);
	assert(ip6[4] == 0x00);
	assert(ip6[5] == 0x00);
	assert(ip6[6] == 0x00);
	assert(ip6[7] == 0x00);
	assert(ip6[8] == 0x00);
	assert(ip6[9] == 0x00);
	assert(ip6[10] == 0x00);
	assert(ip6[11] == 0x00);
	assert(ip6[12] == 0x00);
	assert(ip6[13] == 0x00);
	assert(ip6[14] == 0x00);
	assert(ip6[15] == 0x00);

	input = "::FFFF:1.2.3.4";
	assert(inet_pton(AF_INET6, input, ip6) == 1);
	assert(ip6[0] == 0x00);
	assert(ip6[1] == 0x00);
	assert(ip6[2] == 0x00);
	assert(ip6[3] == 0x00);
	assert(ip6[4] == 0x00);
	assert(ip6[5] == 0x00);
	assert(ip6[6] == 0x00);
	assert(ip6[7] == 0x00);
	assert(ip6[8] == 0x00);
	assert(ip6[9] == 0x00);
	assert(ip6[10] == 0xff);
	assert(ip6[11] == 0xff);
	assert(ip6[12] == 0x01);
	assert(ip6[13] == 0x02);
	assert(ip6[14] == 0x03);
	assert(ip6[15] == 0x04);

	input = "abcd:efg::";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "ab::cd::ef";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "ab:::cd";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "1111:2222:3333:4444:5555:6666:7777";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "1111:2222:3333:4444:5555:6666:7777:8888:9999";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = " 1234::";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "1234:: ";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "::ffff:1.2.3";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "::ffff:1.2.3.4.5";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "::ffff: 1.2.3.4";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "::ffff:1.2.3.04";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "::ffff:0x1.2.3.4";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "12345:::";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "::01234";
	assert(inet_pton(AF_INET6, input, ip6) == 0);

	input = "1.2.3.4";
	assert(inet_pton(AF_INET6, input, ip6) == 0);
#endif

	return 0;
}
