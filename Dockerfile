FROM archlinux AS base
RUN pacman -Syyu --noconfirm
RUN pacman -S --noconfirm git base-devel fakeroot make sudo

FROM base AS test_user
RUN useradd -m docker
RUN echo 'docker:123' | chpasswd
RUN echo 'docker ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/docker
USER docker
WORKDIR  /home/docker

FROM test_user
# grab latest compiled version of aurme
# have some applications installed by default

# remove cache of old versions
RUN sudo pacman -Syu --noconfirm
RUN git clone https://aur.archlinux.org/visual-studio-code-bin.git .cache/aurme/visual-studio-code-bin && \
    cd .cache/aurme/visual-studio-code-bin && \
    makepkg -si --noconfirm
RUN git clone https://aur.archlinux.org/google-chrome.git .cache/aurme/google-chrome && \
    cd .cache/aurme/google-chrome && \
    makepkg -si --noconfirm
RUN git clone https://aur.archlinux.org/firefox-bin.git .cache/aurme/firefox-bin && \
    cd .cache/aurme/firefox-bin && \
    makepkg -si --noconfirm
RUN git clone https://aur.archlinux.org/github-desktop-bin.git .cache/aurme/github-desktop-bin && \
    cd .cache/aurme/github-desktop-bin && \
    makepkg -si --noconfirm
RUN git clone https://aur.archlinux.org/paru.git .cache/aurme/paru && \
    cd .cache/aurme/paru && \
    makepkg -si --noconfirm
RUN git clone https://aur.archlinux.org/yay.git .cache/aurme/yay && \
    cd .cache/aurme/yay && \
    makepkg -si --noconfirm
RUN git clone https://aur.archlinux.org/brave-bin.git .cache/aurme/brave-bin && \
    cd .cache/aurme/brave-bin && \
    git checkout 5df96ce && \
    makepkg -si --noconfirm
RUN git clone https://aur.archlinux.org/android-studio.git .cache/aurme/android-studio && \
    cd .cache/aurme/android-studio && \
    git checkout c67d36d && \
    makepkg -si --noconfirm

COPY target/release/aurme . 
