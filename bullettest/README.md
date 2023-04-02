# bullettest
bullettest is a barebones bulletrl environment modeled after a popular bullet hell game. It's quite basic and doesn't have much variety, but it should be an interesting enough benchmark for reinforcement learning algorithms.

![](imgs/1.png) ![](imgs/2.png) ![](imgs/3.png) ![](imgs/4.png)

For now, only one enemy is supported. There are a couple of randomized movement options:

* Static: The enemy stays in one place
* Sine: The enemy oscillates on the x-axis
* EaseOutExpo: The enemy is interpolated to a new position on a regular interval

And bullet patterns:

* Spiral: Bullets are shot out in a spiral from the enemy
* Direct: Bullets are shot directly towards the player with some spread on a regular interval
* Burst: Bullets are shot towards the player in evenly spaced batches on a regular interval

More complexity will probably be added once I can train a good model on what I have now.