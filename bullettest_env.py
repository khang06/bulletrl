import os
import socket
import struct
import subprocess
import gym
import numpy as np
import cv2

WIDTH = 384
HEIGHT = 448
SCALED_WIDTH = 84
SCALED_HEIGHT = 84
RENDER_SCALE = 4

INPUT_UP = 0b00000001
INPUT_DOWN = 0b00000010
INPUT_LEFT = 0b00000100
INPUT_RIGHT = 0b00001000
INPUT_FOCUS = 0b00010000


def process_image(img):
    img = np.frombuffer(img, dtype=np.uint8).reshape((HEIGHT, WIDTH, 4))  # 1D to 3D
    img = img[:, :, :3]  # Remove alpha
    img = cv2.resize(
        img, (SCALED_HEIGHT, SCALED_WIDTH), interpolation=cv2.INTER_LINEAR
    )  # Scale
    img = np.transpose(img, (2, 0, 1))  # (H, W, C) to (C, H, W)
    return img


class BulletTestEnv(gym.Env):
    metadata = {"render.modes": ["human"]}

    def __init__(self) -> None:
        self.stepped_once = False
        self.action_space = gym.spaces.MultiDiscrete(
            [2, 2, 2, 2, 2]
        )  # up down left right focus
        self.observation_space = gym.spaces.Box(
            low=0, high=255, dtype=np.uint8, shape=(3, SCALED_HEIGHT, SCALED_WIDTH)
        )
        self.obv = None
        self.screen = None

        port = os.getpid() + 4000
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.bind(("127.0.0.1", port))

        subprocess.Popen(["target/release/bullettest.exe", str(port)])
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

    def step(self, action):
        self.stepped_once = True
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
        self.conn.sendall(struct.pack("B", packed_input))
        self.obv = process_image(self.recvfull(WIDTH * HEIGHT * 4))

        self.render()

        reward = struct.unpack("f", self.recvfull(4))[0]
        done = struct.unpack("B", self.recvfull(1))[0] == 1
        return self.obv, reward, done, {}

    def reset(self):
        if self.stepped_once:
            self.conn.sendall(b"\x00")  # Send dummy input
            obv = process_image(self.recvfull(WIDTH * HEIGHT * 4))
            self.recvfull(4)
            self.recvfull(1)
            return obv
        else:
            return np.empty((3, SCALED_HEIGHT, SCALED_WIDTH), dtype=np.uint8)

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

