/* SPDX-License-Identifier: LGPL-2.1+ */
#pragma once

#include <assert.h>
#include <errno.h>
#include <inttypes.h>
#include <stdbool.h>
#include <sys/param.h>
#include <sys/sysmacros.h>
#include <sys/types.h>

#define _printf_(a, b) __attribute__((__format__(printf, a, b)))
#ifdef __clang__
#  define _alloc_(...)
#else
#  define _alloc_(...) __attribute__((__alloc_size__(__VA_ARGS__)))
#endif
#define _sentinel_ __attribute__((__sentinel__))
#define _section_(x) __attribute__((__section__(x)))
#define _used_ __attribute__((__used__))
#define _unused_ __attribute__((__unused__))
#define _destructor_ __attribute__((__destructor__))
#define _pure_ __attribute__((__pure__))
#define _const_ __attribute__((__const__))
#define _deprecated_ __attribute__((__deprecated__))
#define _packed_ __attribute__((__packed__))
#define _malloc_ __attribute__((__malloc__))
#define _weak_ __attribute__((__weak__))
#define _likely_(x) (__builtin_expect(!!(x), 1))
#define _unlikely_(x) (__builtin_expect(!!(x), 0))
#define _public_ __attribute__((__visibility__("default")))
#define _hidden_ __attribute__((__visibility__("hidden")))
#define _weakref_(x) __attribute__((__weakref__(#x)))
#define _align_(x) __attribute__((__aligned__(x)))
#define _alignas_(x) __attribute__((__aligned__(__alignof(x))))
#define _alignptr_ __attribute__((__aligned__(sizeof(void*))))
#define _cleanup_(x) __attribute__((__cleanup__(x)))
#if __GNUC__ >= 7
#define _fallthrough_ __attribute__((__fallthrough__))
#else
#define _fallthrough_
#endif
