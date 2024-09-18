#!/bin/bash

R='\033[1;31m'
G='\033[1;32m'
B='\033[1;34m'
Y='\033[1;33m'
NC='\033[0m' 
BOLD='\033[1m'
TAB='    '

echo -e "${B}Copying${NC} autocomplete script...${NC}"
sudo cp autocomplete/_aurme /usr/share/zsh/site-functions/_aurme

if [ $? -ne 0 ]; then
    echo -e "${R}Failed to copy autocomplete script! Please check your permissions.${NC}"
    exit 1
fi

echo -e "${B}Building${NC} ${BOLD}Aurme${NC} binary..."
cargo build --release

if [ $? -ne 0 ]; then
    echo -e "${R}Build failed! Please check the output for errors.${NC}"
    exit 1
fi

echo -ne "${TAB}${Y}Do you want to install Aurme to /usr/bin? [Y/n]: ${NC}"

read -r answer

if [[ "$answer" =~ ^[Yy]$ || -z "$answer" ]]; then
    echo -e "${TAB}${B}Installing${NC} ${BOLD}Aurme...${NC}"
    sudo cp target/release/aurme /usr/bin/aurme

    if [ $? -ne 0 ]; then
        echo -e "${TAB}${R}Failed to install${NC} ${BOLD}Aurme${NC}! Please check your permissions."
        exit 1
    fi
else
    echo -e "${TAB}${Y}Skipping...${NC}"
fi

echo -e "${G}Finished${NC} installation complete!${NC}\n"
