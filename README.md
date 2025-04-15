# Munal OS

[Screencast_20250215_121948.webm](https://github.com/user-attachments/assets/8cbf8a42-c012-4610-8668-014093efc09d)



Features:
* Custom VirtIO drivers for input, network and full-res display
* TCP stack
* The whole OS is a single EFI binary
* The security model does not implement userspace/kernelspace separation, nor does it put executables in their own virtual address space
* Instead, apps are compiled to WASM and run inside a sandbox
* Available apps:
  * Chronometer
  * 3D demo
  * Python terminal (courtesy of rustpython)
  * Rich text editor
  * A semi-functional web browser
