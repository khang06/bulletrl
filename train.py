import stable_baselines3
from bulletrl_env import BulletTestEnv, Touhou6Env
from stable_baselines3.common.vec_env import DummyVecEnv
from stable_baselines3 import PPO
from stable_baselines3.ppo import CnnPolicy
from stable_baselines3.common.vec_env import VecFrameStack
from stable_baselines3.common.monitor import Monitor

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


def get_wrapped_env(env):
    def f():
        return Monitor(env())

    return f


if __name__ == "__main__":
    env = VecFrameStack(DummyVecEnv([get_wrapped_env(BulletTestEnv) for _ in range(32)]), 2)
    #env = VecFrameStack(DummyVecEnv([get_wrapped_env(Touhou6Env) for _ in range(8)]), 2)
    # env = Touhou6Env()
    model = PPO(
        "CnnPolicy",
        env,
        tensorboard_log="./training/bullettest_again2/",
        batch_size=2048,
        ent_coef=0.01,
        n_steps=2048,
        n_epochs=3,
        learning_rate=1e-4,
        clip_range=0.1,
        #policy_kwargs=dict(net_arch=[128, 128]),
        policy_kwargs=dict(net_arch=[dict(pi=[512], vf=[512])]),
    )
    model.learn(total_timesteps=100_000_000, reset_num_timesteps=False)
    model.save("./training/bullettest_again2")

    """
    model = PPO.load("./training/ppo2_bullettest_final_model")
    model.env = env
    model.tensorboard_log = "./training/ppo2_touhou6_pretrained"
    model.learn(total_timesteps=100_000_000, reset_num_timesteps=True)
    model.save("./training/ppo2_touhou_pretrained_model")
    """
