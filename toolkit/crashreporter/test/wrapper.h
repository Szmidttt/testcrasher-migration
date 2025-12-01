/*
bindgen wrapper.h -o rust/src/phc_bindings.rs --enable-cxx-namespaces --allowlist-function "Rust_.*" --allowlist-function "mozilla::phc::.*" --opaque-type "mozilla::phc::AddrInfo" -- -x c++ -std=c++17
*/

#define MOZ_JEMALLOC_API

namespace mozilla {
namespace phc {
    
// The types here must match the ones in memory/build/PHC.h
class AddrInfo; 

enum PHCState {
  OnlyFree,
  Enabled,
};

} // namespace phc
} // namespace mozilla

extern "C" {
    void Rust_SetPHCState(mozilla::phc::PHCState aState);
    bool Rust_IsPHCAllocation(const void* aPtr, mozilla::phc::AddrInfo* aOutInfo);
}
