#!/bin/bash

# Set error handling
set -e
set -o pipefail
set -u

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
  echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
  echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Error handling function
error_exit() {
  log_error "$1"
  exit 1
}

# Cleanup function
cleanup() {
  log_info "Performing cleanup operations..."
  # Add cleanup logic here if needed
}

# Set cleanup trap
trap cleanup EXIT INT TERM

# Check dependencies
check_dependencies() {
  log_info "Checking dependencies..."
  
  # Check cargo
  if ! command -v cargo &> /dev/null; then
    error_exit "Cargo is not installed. Please install Rust toolchain."
  fi
  
  # Check qemu-system-x86_64
  if ! command -v qemu-system-x86_64 &> /dev/null; then
    error_exit "QEMU is not installed. Please install QEMU system emulator."
  fi
  
  # Check necessary directories
  if [ ! -d "wasm_apps" ]; then
    error_exit "wasm_apps directory does not exist."
  fi
  
  if [ ! -d "kernel" ]; then
    error_exit "kernel directory does not exist."
  fi
  
  if [ ! -d "uefi_firmware" ]; then
    error_exit "uefi_firmware directory does not exist."
  fi
  
  # Check UEFI firmware files
  if [ ! -f "uefi_firmware/code.fd" ]; then
    error_exit "UEFI firmware file code.fd does not exist."
  fi
  
  if [ ! -f "uefi_firmware/vars.fd" ]; then
    error_exit "UEFI firmware file vars.fd does not exist."
  fi
  
  log_success "All dependency checks passed"
}

# Build WASM applications
build_wasm_apps() {
  log_info "Starting to build WASM applications..."
  
  cd wasm_apps/ || error_exit "Cannot enter wasm_apps directory"
  
  local apps=("cube_3d" "chronometer" "terminal" "web_browser" "text_editor")
  
  for app in "${apps[@]}"; do
    log_info "Building $app..."
    
    if [ ! -d "$app" ]; then
      log_warning "Application $app directory does not exist, skipping"
      continue
    fi
    
    cd "$app" || error_exit "Cannot enter $app directory"
    
    if ! cargo build --release; then
      error_exit "Failed to build $app"
    fi
    
    # Check if output file exists
    local wasm_file="target/wasm32-wasip1/release/$app.wasm"
    if [ ! -f "$wasm_file" ]; then
      error_exit "WASM output file for $app does not exist: $wasm_file"
    fi
    
    log_success "$app build completed"
    cd ../ || error_exit "Cannot return to parent directory"
  done
  
  cd ../ || error_exit "Cannot return to project root directory"
  log_success "All WASM applications built successfully"
}

# Embed binary data
embed_binary_data() {
  log_info "Starting to embed binary data..."
  
  # Create kernel/wasm directory
  mkdir -p kernel/wasm/ || error_exit "Cannot create kernel/wasm directory"
  
  local apps=("cube_3d" "chronometer" "terminal" "web_browser" "text_editor")
  
  for app in "${apps[@]}"; do
    local src_file="wasm_apps/$app/target/wasm32-wasip1/release/$app.wasm"
    local dst_file="kernel/wasm/$app.wasm"
    
    if [ ! -f "$src_file" ]; then
      error_exit "Source file does not exist: $src_file"
    fi
    
    log_info "Copying $app.wasm to kernel/wasm directory..."
    if ! cp -uv "$src_file" "$dst_file"; then
      error_exit "Failed to copy file: $src_file -> $dst_file"
    fi
    
    log_success "$app.wasm embedded successfully"
  done
  
  log_success "All binary data embedded successfully"
}

# Build kernel
build_kernel() {
  log_info "Starting to build kernel..."
  
  cd kernel/ || error_exit "Cannot enter kernel directory"
  
  if ! cargo build --release; then
    error_exit "Kernel build failed"
  fi
  
  # Check kernel output file
  local kernel_file="target/x86_64-unknown-uefi/release/kernel.efi"
  if [ ! -f "$kernel_file" ]; then
    error_exit "Kernel output file does not exist: $kernel_file"
  fi
  
  log_success "Kernel build completed"
  cd ../ || error_exit "Cannot return to project root directory"
}

# Run QEMU
run_qemu() {
  log_info "Preparing to run QEMU..."
  
  # Create ESP directory structure
  mkdir -p esp/efi/boot/ || error_exit "Cannot create ESP directory structure"
  
  # Copy kernel file
  local kernel_src="kernel/target/x86_64-unknown-uefi/release/kernel.efi"
  local kernel_dst="esp/efi/boot/bootx64.efi"
  
  if [ ! -f "$kernel_src" ]; then
    error_exit "Kernel file does not exist: $kernel_src"
  fi
  
  log_info "Copying kernel file to ESP directory..."
  if ! cp -uv "$kernel_src" "$kernel_dst"; then
    error_exit "Failed to copy kernel file"
  fi
  
  log_success "QEMU environment preparation completed"
  log_info "Starting QEMU emulator..."
  
  # Check KVM support
  if [ ! -e "/dev/kvm" ]; then
    log_warning "KVM is not available, will use software emulation (slower performance)"
    kvm_enabled=""
  else
    kvm_enabled="-enable-kvm"
  fi
  
  # Run QEMU
  qemu-system-x86_64 \
    $kvm_enabled \
    -m 1G \
    -rtc base=utc \
    -display sdl \
    -drive if=pflash,format=raw,readonly=on,file=uefi_firmware/code.fd \
    -drive if=pflash,format=raw,readonly=on,file=uefi_firmware/vars.fd \
    -drive format=raw,file=fat:rw:esp \
    -device virtio-keyboard \
    -device virtio-mouse \
    -device virtio-net-pci,netdev=network0 -netdev user,id=network0 \
    -vga virtio \
    -serial stdio || error_exit "QEMU execution failed"
}

# Show help information
show_help() {
  echo "Munal OS Build and Run Script"
  echo ""
  echo "Usage: $0 [options]"
  echo ""
  echo "Options:"
  echo "  -h, --help     Show this help information"
  echo "  -b, --build    Build only, do not run QEMU"
  echo "  -r, --run      Run QEMU only (requires built files)"
  echo "  -c, --clean    Clean build files"
  echo ""
  echo "Examples:"
  echo "  $0              # Build and run"
  echo "  $0 --build      # Build only"
  echo "  $0 --run        # Run only"
  echo "  $0 --clean      # Clean build files"
}

# Clean build files
clean_build() {
  log_info "Starting to clean build files..."
  
  # Clean WASM applications
  if [ -d "wasm_apps" ]; then
    log_info "Cleaning WASM application build files..."
    find wasm_apps -name "target" -type d -exec rm -rf {} + 2>/dev/null || true
  fi
  
  # Clean kernel build
  if [ -d "kernel" ]; then
    log_info "Cleaning kernel build files..."
    if [ -d "kernel/target" ]; then
      rm -rf kernel/target
    fi
  fi
  
  # Clean embedded data
  if [ -d "kernel/wasm" ]; then
    log_info "Cleaning embedded data files..."
    find kernel/wasm -name "*.wasm" -type f -delete 2>/dev/null || true
  fi
  
  # Clean ESP directory
  if [ -d "esp" ]; then
    log_info "Cleaning ESP directory..."
    rm -rf esp
  fi
  
  log_success "Cleanup completed"
}

# Main function
main() {
  local build_only=false
  local run_only=false
  local clean_only=false
  
  # Parse command line arguments
  while [[ $# -gt 0 ]]; do
    case $1 in
      -h|--help)
        show_help
        exit 0
        ;;
      -b|--build)
        build_only=true
        shift
        ;;
      -r|--run)
        run_only=true
        shift
        ;;
      -c|--clean)
        clean_only=true
        shift
        ;;
      *)
        log_error "Unknown option: $1"
        show_help
        exit 1
        ;;
    esac
  done
  
  # Execute corresponding operations
  if [ "$clean_only" = true ]; then
    clean_build
    exit 0
  fi
  
  if [ "$run_only" = true ]; then
    log_info "Run only mode..."
    run_qemu
    exit 0
  fi
  
  # Check dependencies
  check_dependencies
  
  # Build WASM applications
  build_wasm_apps
  
  # Embed binary data
  embed_binary_data
  
  # Build kernel
  build_kernel
  
  if [ "$build_only" = false ]; then
    # Run QEMU
    run_qemu
  else
    log_success "Build completed"
  fi
}

# Run main function
main "$@"
