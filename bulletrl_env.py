import os
import socket
import struct
import subprocess
import gymnasium as gym
import numpy as np
import cv2

WIDTH = 384
HEIGHT = 448
SCALED_WIDTH = 84
SCALED_HEIGHT = 84
RENDER_SCALE = 4

INPUT_UP    = 0b00000001
INPUT_DOWN  = 0b00000010
INPUT_LEFT  = 0b00000100
INPUT_RIGHT = 0b00001000
INPUT_FOCUS = 0b00010000

# Since I'm doing manual TCP communication, it's possible that it might desync due to a programming error
# This should catch that
TCP_SENTINEL = 0x1337BEEF


def process_image(img):
    img = np.frombuffer(img, dtype=np.uint8).reshape((HEIGHT, WIDTH, 4))  # 1D to 3D
    img = img[:, :, :3]  # Remove alpha
    img = cv2.resize(
        img, (SCALED_HEIGHT, SCALED_WIDTH), interpolation=cv2.INTER_LINEAR
    )  # Scale
    img = np.transpose(img, (2, 0, 1))  # (H, W, C) to (C, H, W)
    return img


class BulletRLEnv(gym.Env):
    metadata = {"render.modes": ["human"]}

    def __init__(self) -> None:
        if self.cmdline_base is None:
            raise Exception("You shouldn't directly construct a BulletRLEnv")

        self.stepped_once = False
        #self.action_space = gym.spaces.MultiDiscrete(
        #    [2, 2, 2, 2, 2]
        #)  # up down left right focus
        self.action_space = gym.spaces.Discrete(32)
        self.observation_space = gym.spaces.Box(
            low=0, high=255, dtype=np.uint8, shape=(3, SCALED_HEIGHT, SCALED_WIDTH)
        )
        self.obv = None
        self.screen = None

        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.bind(("", 0))
        port = self.socket.getsockname()[1]

        subprocess.Popen(self.cmdline_base + [str(port)])
        print("Waiting for client...")
        self.socket.listen(1)
        (self.conn, _) = self.socket.accept()

        print("Init done")

    def recvfull(self, size):
        read = b""
        left = size
        while len(read) != size:
            last_size = len(read)
            read += self.conn.recv(left)
            left -= len(read) - last_size
        return read

    def send_input(self, input):
        self.conn.sendall(struct.pack("I", TCP_SENTINEL))
        self.conn.sendall(struct.pack("B", input))

    def recv_obv(self):
        if struct.unpack("I", self.recvfull(4))[0] != TCP_SENTINEL:
            raise Exception("TCP desync check failed!")

        return (
            process_image(self.recvfull(WIDTH * HEIGHT * 4)),
            struct.unpack("f", self.recvfull(4))[0],
            struct.unpack("B", self.recvfull(1))[0] == 1,
        )

    def step(self, action):
        self.stepped_once = True
        '''
        packed_input = 0
        if action[0] == 1:
            packed_input |= INPUT_UP
        if action[1] == 1:
            packed_input |= INPUT_DOWN
        if action[2] == 1:
            packed_input |= INPUT_LEFT
        if action[3] == 1:
            packed_input |= INPUT_RIGHT
        if action[4] == 1:
            packed_input |= INPUT_FOCUS
        self.send_input(packed_input)
        '''
        self.send_input(action)

        self.obv, reward, done = self.recv_obv()

        # self.render()
        return self.obv, reward, done, False, {}

    def reset(self):
        if self.stepped_once:
            self.send_input(0)  # Send dummy input
            obv, _reward, _done = self.recv_obv()
            return obv, {}
        else:
            return np.empty((3, SCALED_HEIGHT, SCALED_WIDTH), dtype=np.uint8), {}

    def render(self, mode="human"):
        import pygame

        if self.screen is None:
            pygame.init()
            pygame.display.init()
            self.screen = pygame.display.set_mode(
                (SCALED_WIDTH * RENDER_SCALE, SCALED_HEIGHT * RENDER_SCALE)
            )
        self.screen.blit(
            pygame.surfarray.make_surface(
                cv2.resize(
                    np.transpose(self.obv, (2, 1, 0)),
                    (SCALED_HEIGHT * RENDER_SCALE, SCALED_WIDTH * RENDER_SCALE),
                    interpolation=cv2.INTER_NEAREST,
                )
            ),
            (0, 0),
        )
        pygame.event.pump()
        pygame.display.flip()


class BulletTestEnv(BulletRLEnv):
    def __init__(self) -> None:
        binary = "bullettest.exe" if os.name == "nt" else "bullettest"
        self.cmdline_base = [f"bullettest/target/release/{binary}"]
        super().__init__()


class Touhou6Env(BulletRLEnv):
    def __init__(self) -> None:
        # TODO: Don't hard code paths
        self.cmdline_base = [
            "tinyinjector32.exe",
            "bulletrl_th6\\target\\i686-pc-windows-msvc\\release\\bulletrl_th6.dll",
            "d:\\Games\\touhou\\EoSD-AI-2\\th06e.exe",
        ]
        super().__init__()
