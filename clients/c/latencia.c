#define _POSIX_C_SOURCE 200809L
// Latencia de ida y vuelta (RTT) entre dos procesos: anillo shm vs pipe vs socket Unix.
#include "ring.h"
#include <sys/socket.h>
#include <sys/wait.h>
static uint64_t ns(){struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return (uint64_t)t.tv_sec*1000000000ull+t.tv_nsec;}
static int cmp(const void*a,const void*b){uint64_t x=*(uint64_t*)a,y=*(uint64_t*)b;return x<y?-1:x>y;}
static void rep(const char*k,uint64_t*v,int n,int err){qsort(v,n,8,cmp);
 printf("%-12s n=%d p50=%.2fus p99=%.2fus p99.9=%.2fus max=%.2fus errores=%d\n",k,n,v[n/2]/1e3,v[n*99/100]/1e3,v[n*999/1000]/1e3,v[n-1]/1e3,err);}
#define N 200000
#define W 10000
int main(){static uint64_t v[N];
 /* anillo: ping a->b, pong b->a */
 Ring a=ring_open("lat_a",1024,1), b=ring_open("lat_b",1024,1);
 pid_t p=fork();
 if(!p){Ring a2=ring_open("lat_a",1024,0),b2=ring_open("lat_b",1024,0);uint64_t x;
   for(int i=0;i<N+W;i++){while(!ring_pop(&a2,&x,1));x++;while(!ring_push(&b2,&x,1));}_exit(0);}
 int err=0;
 for(int i=0;i<N+W;i++){uint64_t x=i,y;uint64_t t=ns();while(!ring_push(&a,&x,1));while(!ring_pop(&b,&y,1));uint64_t d=ns()-t;if(y!=(uint64_t)i+1)err++;if(i>=W)v[i-W]=d;}
 waitpid(p,0,0);rep("anillo_shm",v,N,err);shm_unlink("/lat_a");shm_unlink("/lat_b");
 for(int k=0;k<2;k++){int f1[2],f2[2];
  if(k==0){if(pipe(f1)||pipe(f2))return 1;}else{if(socketpair(AF_UNIX,SOCK_STREAM,0,f1))return 1;f2[0]=f1[0];f2[1]=f1[1];}
  p=fork();
  if(!p){int rd=k?f1[1]:f1[0],wr=k?f1[1]:f2[1];uint64_t x;for(int i=0;i<N+W;i++){if(read(rd,&x,8)!=8)_exit(1);x++;if(write(wr,&x,8)!=8)_exit(1);}_exit(0);}
  int wr=k?f1[0]:f1[1],rd=k?f1[0]:f2[0];err=0;
  for(int i=0;i<N+W;i++){uint64_t x=i,y;uint64_t t=ns();if(write(wr,&x,8)!=8)return 1;if(read(rd,&y,8)!=8)return 1;uint64_t d=ns()-t;if(y!=(uint64_t)i+1)err++;if(i>=W)v[i-W]=d;}
  waitpid(p,0,0);rep(k?"socket_unix":"pipe",v,N,err);}
}
