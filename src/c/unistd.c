#include <stdarg.h>
#include <stddef.h>

// TODO: Can be implemented in rust when cbindgen supports "..." syntax

int execv(const char *path, char *const *argv);

int execl(const char *path, const char* argv0, ...)
{
	int argc;
	va_list ap;
	va_start(ap, argv0);
	for (argc = 1; va_arg(ap, const char*); argc++);
	va_end(ap);
	{
		int i;
		char *argv[argc+1];
		va_start(ap, argv0);
		argv[0] = (char *)argv0;
		for (i = 1; i < argc; i++) {
			argv[i] = va_arg(ap, char *);
		}
		argv[i] = NULL;
		va_end(ap);
		return execv(path, argv);
	}
}

int execve(const char *path, char *const *argv, char *const *envp);

int execle(const char *path, const char* argv0, ...)
{
	int argc;
	va_list ap;
	va_start(ap, argv0);
	for (argc = 1; va_arg(ap, const char *); argc++);
	va_end(ap);
	{
		int i;
		char *argv[argc+1];
		char **envp;
		va_start(ap, argv0);
		argv[0] = (char *)argv0;
		for (i = 1; i <= argc; i++) {
			argv[i] = va_arg(ap, char *);
		}
		envp = va_arg(ap, char **);
		va_end(ap);
		return execve(path, argv, envp);
	}
}

int execvp(const char *file, char *const *argv);

int execlp(const char *file, const char* argv0, ...)
{
	int argc;
	va_list ap;
	va_start(ap, argv0);
	for (argc = 1; va_arg(ap, const char*); argc++);
	va_end(ap);
	{
		int i;
		char *argv[argc+1];
		va_start(ap, argv0);
		argv[0] = (char *)argv0;
		for (i = 1; i < argc; i++) {
			argv[i] = va_arg(ap, char *);
		}
		argv[i] = NULL;
		va_end(ap);
		return execvp(file, argv);
	}
}
