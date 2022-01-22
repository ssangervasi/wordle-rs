# wordle-rs

[Wordle](https://www.powerlanguage.co.uk/wordle/) in your terminal.

## Demo!
![Demo gif](https://thumbs.gfycat.com/AdoredForcefulIrishsetter-size_restricted.gif)

## Now supports windows!
![Powershell](https://i.imgur.com/IfswepI.png)

## Usage

```
OPTIONS:
  -a, --ascii                  ASCII mode. Good for windows or if you're color blind like me :D
                                   x : Miss (not in word)
                                   ~ : Close (in word, wrong position)
                                   @ : Match (in word at this position)
  -h, --help                   Print help information
  -l, --inline                 Play line-by-line instead of interactively.
  -n, --word-len <WORD_LEN>    I dare you to try 2- or 10-letter words. [default: 5]
  -u, --unicode
  -V, --version                Print version information
  -w, --word <WORD>            The word you want to guess. (Maybe you set this up for someone
                               else? Just testing things out?)
```