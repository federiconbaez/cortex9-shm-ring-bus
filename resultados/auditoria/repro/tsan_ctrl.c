// TSan sobre clients/c/ring.h: productor y consumidor en hilos sobre UNA sola proyección (TSan no ve entre proyecciones).
#define _POSIX_C_SOURCE 200809L
#include "ring_relaxed.h"
#include <pthread.h>
#define N 2000000ull
static Ring P, C;
static void *prod(void *a){(void)a;uint64_t buf[37];uint64_t next=0;
 while(next<N){size_t k=37;if(N-next<k)k=N-next;for(size_t i=0;i<k;i++)buf[i]=next+i;size_t off=0;while(off<k)off+=ring_push(&P,buf+off,k-off);next+=k;}return 0;}
int main(void){
 P=ring_open("audit10_tsan",1024,1); C=P; /* misma base, estados locales separados */
 pthread_t t; pthread_create(&t,0,prod,0);
 uint64_t out[53],expect=0,err=0;
 while(expect<N){size_t n=ring_pop(&C,out,53);for(size_t i=0;i<n;i++){if(out[i]!=expect)err++;expect++;}}
 pthread_join(t,0); ring_unlink("audit10_tsan");
 printf("tsan ring.h: recibidos=%llu errores=%llu\n",(unsigned long long)expect,(unsigned long long)err); return err!=0;}
