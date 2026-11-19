// Línea base informativa: pipe con O_NONBLOCK y espera activa (mismo costo de CPU que el anillo).
#define _GNU_SOURCE
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#include <fcntl.h>
#include <time.h>
#include <sys/wait.h>
static uint64_t ns(){struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return (uint64_t)t.tv_sec*1000000000ull+t.tv_nsec;}
static int cmp(const void*a,const void*b){uint64_t x=*(uint64_t*)a,y=*(uint64_t*)b;return x<y?-1:x>y;}
#define N 200000
#define W 10000
static void rd(int fd,uint64_t*x){for(;;){ssize_t k=read(fd,x,8);if(k==8)return;if(k>0)exit(3);}}
int main(){static uint64_t v[N];int f1[2],f2[2];
 if(pipe2(f1,O_NONBLOCK)||pipe2(f2,O_NONBLOCK))return 1;
 pid_t p=fork();
 if(!p){uint64_t x;for(int i=0;i<N+W;i++){rd(f1[0],&x);x++;if(write(f2[1],&x,8)!=8)_exit(1);}_exit(0);}
 int err=0;
 for(int i=0;i<N+W;i++){uint64_t x=i,y;uint64_t t=ns();if(write(f1[1],&x,8)!=8)return 1;rd(f2[0],&y);uint64_t d=ns()-t;if(y!=(uint64_t)i+1)err++;if(i>=W)v[i-W]=d;}
 waitpid(p,0,0);qsort(v,N,8,cmp);
 printf("pipe_spin    n=%d p50=%.2fus p99=%.2fus p99.9=%.2fus max=%.2fus errores=%d\n",N,v[N/2]/1e3,v[N*99/100]/1e3,v[N*999/1000]/1e3,v[N-1]/1e3,err);}
