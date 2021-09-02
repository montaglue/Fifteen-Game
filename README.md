# Fifteen-Game

A discord bot for [Fifteen Game] 

[Fifteen Game]: https://en.wikipedia.org/wiki/15_puzzle

## Commands


### Game life time

```
~start
```
Starts a new game in this chat room. This should be first command. In new chats.

```
~refresh
```
Set game positon to default one.

```
~maze
```
Set geme positon to random one, but always solvable.

### Empty cell movements

```
~up
~down
~right
~left
```
Moves the empty cell to the chosen direction.

### Solution search

```
~solution
```

Bot find and print the optimal solution for the current board position.

## Limits

All games are deleted after bot shutdown.

## Solution searching algorithm

This implementation uses [A*].

### Heuristic
I calculate manhattan distance between cell and this cell in solved position.
Then sum up distances for all cells exept the empty one.

### Optimizations

- Meet in the midle
- Bit magic


[A*]: https://en.wikipedia.org/wiki/A*_search_algorithm 