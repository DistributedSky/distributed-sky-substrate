# distrubuted-sky-substrate

Experimental repo for Distributed Sky Substrate project

## Overview

Distributed Sky (DS) blockchain - is a try to build a system with public and provable security in flight routes registration fr UAVs, avoiding flight dispatching errors at the moment of registration, and to build a simple public insfrastructure for exchanging flight information between heterogenous systems in different countries in a non-conflicting way.
Onchain validation of flight routes allows pilots to avoid problems with compatibility of systems, route registration ordering, make any bugs in algorithms publicly verifiable.
Technically - it's a blockchain, allowing to REGISTRARs set "green" and "red" zones, where flights are permit or forbidden.
PILOTs are registering flight routes for their UAVs. These flight routes must be in allowed "green" zones and can't intersect "red" zones, set before route registration. This check is performed onchain, by publicly verifiable, open-source code with ability to trace every transaction and continiously improve this code to ideal, being able to be analysed and upgraded by any party or company, wishing to participate in development and safety of protocol.
  
Main problem of such onchan-based checks is the complexity of search through spatial data and computation restrictions for blockchain transactions. The main requirement for onchain validation is to perform needed checks in O(1) complexity by CPU, memory, storage and network.

## [WARNING] 

Current code is a proof-of-concept, demonstating ability to algorithmically solve problem of complicated checks, performed onchain. Many future changes in protocol are required to make this sytem production-ready.


## Runtime functions

Current proof-of-concept runtime includes functions:

- setting account role by ADMIN (for creating REGISTRARs)
- register PILOT by REGISTRAR (creating record about PILOT and his license in public registry)
- registration of UAV by PILOT (creating record about UAV in public registry)
- setting "flight zone" by REGISTRAR (set area on map, in which setting routes is allowed)
- setting "red" areas inside flight zone by REGISTRAR (set area on map, that cannot be intersected by any route)
- setting route (by checking intersections with zones and firing blockchain Event with flight route params)

# Build and run

```bash
# install rust and all required dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# clone project from repo
git clone https://github.com/DistributedSky/distributed-sky-substrate.git
cd distributed-sky-substrate

# add build target "wasm32-unknown-unknown" to build runtime
rustup target add wasm32-unknown-unknown

# build release
cargo build --release

# run node in "dev" mode
./target/release/node-dsky --dev
```

## Future improvements

Currently only basic functionality is shown, proving that onchain validation of routes is possible in current blockchain environment. Future additions to this runtime include:

- free shapes for "red zones" 
- flight routes with may waypoints
- many other stuff...


