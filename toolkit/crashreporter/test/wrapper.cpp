#include "PHC.h" 

extern "C" {
    
    void Rust_SetPHCState(mozilla::phc::PHCState aState) {
        mozilla::phc::SetPHCState(aState);
    }

    bool Rust_IsPHCAllocation(const void* aPtr, mozilla::phc::AddrInfo* aOutInfo) {
        return mozilla::phc::IsPHCAllocation(aPtr, aOutInfo);
    }
}