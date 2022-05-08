import stable_baselines3
from bullettest_env import BulletTestEnv
from stable_baselines3.common.vec_env import SubprocVecEnv
from stable_baselines3 import PPO
from stable_baselines3.ppo import CnnPolicy
from stable_baselines3.common.vec_env import VecFrameStack
import pygame

if __name__ == "__main__":
    env = VecFrameStack(SubprocVecEnv([BulletTestEnv for _ in range(1)]), 2)
    model = PPO.load("./training/ppo2_bullettest_final_model")
    clock = pygame.time.Clock()
    obs = env.reset()
    while True:
        action, _states = model.predict(obs)
        obs, rewards, dones, info = env.step(action)
        # env.render()
        clock.tick(15)
