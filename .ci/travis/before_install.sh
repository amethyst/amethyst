#!/usr/bin/env bash

set -exuo pipefail

MDBOOK_VERSION="v0.2.1"
SCCACHE_VERSION="0.2.7"

install_libsdl2(){
  case ${TRAVIS_OS_NAME} in
  windows)
    wget --no-check-certificate https://www.libsdl.org/release/SDL2-devel-2.0.8-VC.zip
    7z x SDL2-devel-2.0.8-VC.zip
    cp SDL2-2.0.8/lib/x64/*.lib ${HOME}/.rustup/toolchains/${TRAVIS_RUST_VERSION}-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/lib
    cp SDL2-2.0.8/lib/x64/*.dll .
    rm SDL2-devel-2.0.8-VC.zip
    ;;
  osx)
    brew update && brew install sdl2
    ;;
  linux)
    # This is handled by Travis CI's `apt` addon
    ;;
  esac
}

install_mdbook(){
  case ${TRAVIS_OS_NAME} in
  linux)
    export MDBOOK_RELEASE="${MDBOOK_VERSION}/mdbook-${MDBOOK_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
    curl -L -o mdbook.tar.gz https://github.com/rust-lang-nursery/mdBook/releases/download/${MDBOOK_RELEASE}
    tar -xvf mdbook.tar.gz -C ./
    rm mdbook.tar.gz
    ;;
  esac
}

install_sccache(){
  case ${TRAVIS_OS_NAME} in
  windows)
    SCCACHE_TOOLCHAIN="x86_64-pc-windows-msvc"
    SCCACHE_EXECUTABLE_NAME="sccache.exe" 
    ;;
  osx)
    SCCACHE_TOOLCHAIN="x86_64-apple-darwin"
    SCCACHE_EXECUTABLE_NAME="sccache" 
    ;;
  linux)
    SCCACHE_TOOLCHAIN="x86_64-unknown-linux-musl"
    SCCACHE_EXECUTABLE_NAME="sccache" 
    ;;
  esac

  SCCACHE_RELEASE="sccache-${SCCACHE_VERSION}-${SCCACHE_TOOLCHAIN}"
  SCCACHE_ARCHIVE_NAME="${SCCACHE_RELEASE}.tar.gz"
  SCCACHE_URL="https://github.com/mozilla/sccache/releases/download/${SCCACHE_VERSION}/${SCCACHE_ARCHIVE_NAME}"

  curl -L -O ${SCCACHE_URL} 
  tar -xvf ${SCCACHE_ARCHIVE_NAME} -C ./ --strip=1 ${SCCACHE_RELEASE}/${SCCACHE_EXECUTABLE_NAME}
  rm ${SCCACHE_ARCHIVE_NAME}
}

install_libsdl2
install_mdbook
install_sccache

set +exuo pipefail
