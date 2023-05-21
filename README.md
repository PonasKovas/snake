# snake
A simple light-weight terminal snake game

## Installation

### AUR

This game is available on [AUR](https://wiki.archlinux.org/index.php/Arch_User_Repository).

```bash
git clone https://aur.archlinux.org/snake.git
cd snake
makepkg -si
```

## Running

```bash
snake
```
The game will start right away.

To quit, press Q.

To pause/play, press Escape.

To start at a given level in order to skip the slow start of the game, pass a second integer argument determining the desired level. `snake 10`

When you die, the game exits, so if you want to play again, launch it again.

To clear the high score, delete the file at `$HOME/.snake`.
