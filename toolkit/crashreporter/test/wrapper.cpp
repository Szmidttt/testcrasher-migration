#include "PHC.h"
#include "mozilla/mozalloc.h"
#include "mozilla/Assertions.h"

extern "C" {

void Rust_SetPHCState(mozilla::phc::PHCState aState) {
  mozilla::phc::SetPHCState(aState);
}

bool Rust_IsPHCAllocation(const void* aPtr, mozilla::phc::AddrInfo* aOutInfo) {
  return mozilla::phc::IsPHCAllocation(aPtr, aOutInfo);
}

void *Rust_moz_xmalloc(size_t size){
    return moz_xmalloc(size);
}

void Rust_MOZ_CRASH(){
    MOZ_CRASH();
}

/*
 * This pure virtual call example is from MSDN
 */
class A;

void fcn(A*);

class A {
 public:
  virtual void f() = 0;
  A() { fcn(this); }
};

class B : A {
  void f() override {}

 public:
  void use() {}
};

void fcn(A* p) { p->f(); }

void PureVirtualCall() {
  // generates a pure virtual function call
  B b;
  b.use();  // make sure b's actually used
}

}