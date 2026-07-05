# Multi Uni Kernel
Here is the vision of Uni Kernel:

1) At boot time, a bootloader (~300 lines) runs and prompts the user what unikernels they would like to start. Each unikernel is not an operating system -- it is an application that runs directly on bare metal, akin to some of the earliest software.

2) A bootloader also executes a hypervisor instruction set (~100 lines) into RAM, reserving a very small portion for that purpose (more on that later) A table is loaded into the cpu to launch the code on an interrupt signal (this is supported by native CPU architecture), and does not need any other code, it is a passive code execution trigger. Or in otherwords: it sets up a keystroke to fire the code (this will become magical later on)

3) Finally, the bootloader starts the unikernels in parallel, giving each unikernel a single core and dividing the ram evenly between them.  Every unikernel (think like super deep docker container to the bare metal layer) is only aware of itself, it literally cant in anyway screw up any other app (unikernel) and only knows, cares, and tracks its own resources. -- this prevents a HOST of hardware crashes, failures, and avoid complex kernel logic.  Infact it is IMPOSSIBLE for one unikernel to crash another, meaning each app (since app = kernel = OS) is fully secure and safe from one another.

3) Now with each unikernel running in parallel and a keystroke registered something magical happens: you can switch between each unikernel (say with alt + tab) cycling through the running threads.  You might have a spreadsheet app, a terminal editor app, a slim proper linux terminal (as even full kernels can run -- any OS can run if it can run on one core and limited RAM).  

So the end result: You can then use each of these like they are applications in and of themselves.  It inverts OS design, the OS is no longer an OS but a wrapper of an application like a docker shell, but unlike (and even better than docker) the shell does not need to align to the same OS as it all runs on bare metal.  An OS that runs on your hardware, any kernel, anything could be put on one of the cores (if it could support the resource limitations).  It frees you from OS ecosystems entirely.

So it goes from something like this:


| App 1 | App 2 | App 3 | App 4 |
_______________________________
Monolithic OS that controls you, everything 
and screws up resource assignment to:
------- ALL CORES ---------------------
-------ENTIRE MEMORY POOL -------------


TO:


| OS/App 1 | OS/App 2 | OS/App 3 | OS/App 4 |
| CORE 1   | CORE 2   | CORE 3   | CORE 4   |
| 2GB      | 2GB      |  2GB     |  2GB

^^ No middleman, perfect assignment of resources from boot,
with the monolothic OS moved into a micro-OS (unikernel is the proper term) that
can be different for each app and lives as part of the app -- built into it, and frees you of control.

Think about it like running N64 cartridges but rather than a physical switch of carts and wait time it is an instantaneous CPU level switch on alt+tab switch since they are all held in reserved memory.

Sort of like if you could have the power of an early 2000s computer but in 8-32 (depending on your thread and core count) totally independent machines.  It wouldn't be hard to allocate 2 cores to each if needed, it could be a decision of the boot loader.

