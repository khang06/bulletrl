# bulletrl
bulletrl is a simple [OpenAI Gym](https://github.com/openai/gym) environment for training reinforcement learning models on bullet hell games. It's mostly developed around Touhou Project games for now.

Currently, the supported games are:
* bullettest (barebones test environment)
* Touhou Koumakyou ~ the Embodiment of Scarlet Devil v1.02h

Ports to other games are welcome.

# TODO
- [x] Train a working model for bullettest: Trained for 100m steps, download it [here](https://files.catbox.moe/qeggsn.zip)
- [ ] Refactor common interface code into a library
- [x] Port/rewrite [th6hook](https://github.com/khang06/th6hook)
- [ ] Support for manual environment resets
- [ ] Configurability