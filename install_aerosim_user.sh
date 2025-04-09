#!/bin/bash
# AeroSim User Installation Script - Linux

# -------------------------------------------------------------
# Setup Colors using tput (if stdout is a terminal)
# -------------------------------------------------------------
if [ -t 1 ]; then
    GREEN=$(tput setaf 2)
    YELLOW=$(tput setaf 3)
    RED=$(tput setaf 1)
    CYAN=$(tput setaf 6)
    MAGENTA=$(tput setaf 5)
    RESET=$(tput sgr0)
else
    GREEN=""
    YELLOW=""
    RED=""
    CYAN=""
    MAGENTA=""
    RESET=""
fi

# -------------------------------------------------------------
# Clear Screen and Display Banner and Intro Text
# -------------------------------------------------------------
clear
echo -e "${MAGENTA}          ___                  _____ _          ${NC}"
echo -e "${MAGENTA}         /   | ___  _________ / ___/(_)___ ___   ${NC}"
echo -e "${MAGENTA}        / /| |/ _ \/ ___/ __ \\__ \/ / __ \`__ \  ${NC}"
echo -e "${MAGENTA}       / ___ /  __/ /  / /_/ /__/ / / / / / / /  ${NC}"
echo -e "${MAGENTA}      /_/  |_\___/_/   \____/____/_/_/ /_/ /_/   ${NC}"
echo ""

echo "Welcome to the AeroSim User Installer!"
echo
echo "${YELLOW}IMPORTANT INFORMATION:${RESET}"
echo "- Please review the user documentation before proceeding."
echo "- For detailed instructions, visit:"
echo "  https://github.com/aerosim-open/refactor-aerosim/tree/dev/docs"
echo

# -------------------------------------------------------------
# Wait for the User to Press Enter to Exit
# -------------------------------------------------------------
read -rp "Press Enter to exit..."
