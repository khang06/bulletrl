import stable_baselines3
from bullettest_env import BulletTestEnv
from stable_baselines3.common.vec_env import SubprocVecEnv
from stable_baselines3 import PPO
from stable_baselines3.ppo import CnnPolicy
from stable_baselines3.common.vec_env import VecFrameStack

# Here for testing, taken from openai-gym source
class RandomAgent(object):
    """The world's simplest agent!"""

    def __init__(self, action_space):
        self.action_space = action_space

    def act(self, observation, reward, done):
        return self.action_space.sample()


"""
if __name__ == "__main__":
    env = BulletTestEnv()
    agent = RandomAgent(env.action_space)
    reward = 0
    done = False
    for i in range(1000):
        ob = env.reset()
        while True:
            action = agent.act(ob, reward, done)
            ob, reward, done, _ = env.step(action)
            env.render()
            if done:
                break
        print("env done")
    env.close()
"""

"""
if __name__ == "__main__":
    from stable_baselines3.common.env_checker import check_env

    check_env(BulletTestEnv())
"""


def linear_schedule(initial_value):
    def func(progress):
        return progress * initial_value

    return func


if __name__ == "__main__":
    env = VecFrameStack(SubprocVecEnv([BulletTestEnv for _ in range(8)]), 2)
    # env = BulletTestEnv()
    model = PPO(
        "CnnPolicy",
        env,
        tensorboard_log="./training/ppo2_bullettest/",
        batch_size=1024,
        ent_coef=0.005,
        n_steps=2048,
        n_epochs=3,
        learning_rate=5e-5,
        clip_range=0.1,
        # policy_kwargs=dict(net_arch=[64, 64]),
        policy_kwargs=dict(net_arch=[dict(pi=[32, 32], vf=[32, 32])]),
    )
    model.learn(total_timesteps=100_000_000, reset_num_timesteps=False)
    model.save("./training/ppo2_bullettest_model")