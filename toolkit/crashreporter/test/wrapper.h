/*
bindgen wrapper.h -o rust/src/phc_bindings.rs --enable-cxx-namespaces --allowlist-function "mozilla::phc::.*" --opaque-type "mozilla::phc::AddrInfo" -- -x c++ -std=c++17
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

MOZ_JEMALLOC_API void SetPHCState(PHCState aState);

MOZ_JEMALLOC_API bool IsPHCAllocation(const void*, AddrInfo*);

} // namespace phc
} // namespace mozilla