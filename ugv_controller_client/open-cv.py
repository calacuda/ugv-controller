from json import dumps

import cv2
import pygame
from requests import post

from lib_ctrlr.config import *
from lib_ctrlr.controls import *
from lib_ctrlr.logger import log

# The RTSP URL (replace with your stream's URL)
# A typical format is 'rtsp://username:password@ip_address:port/stream_path'
# rtsp_url = "rtsp://your_stream_url"
# rtsp_url = "rtsp://localhost:8554/main"
ip = "192.168.1.146"
rtsp_url = f"rtsp://{ip}:8554/main"
api_url = f"http://{ip}:5000/send_command"
# rtsp_url = "rtsp://192.168.50.5:8554/main"

# Create a VideoCapture object
cap = cv2.VideoCapture(rtsp_url)

# Check if the connection was successful
if not cap.isOpened():
    print("Error: Could not open video stream.")
    exit()

print("Connection successful. Reading stream...")

pygame.init()
pygame.font.init()
s_width, s_height = 1280, 720
# width, height = 1920, 1080
# width, height = 640, 480
screen = pygame.display.set_mode((s_width, s_height))
pygame.display.set_caption("Rendered to Pygame")
font = pygame.font.SysFont("Arial", 24)
clock = pygame.time.Clock()
fps_text = "FPS = ?"
running = True
log.info(f"starting loop")
controller = Buttons()
joy = None

last_move = (0, 0)

# Read and display frames in a loop
while True:
    current_fps = clock.get_fps()

    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            log.debug("Quit")
            break
        elif event.type == pygame.JOYDEVICEADDED:
            joy = pygame.joystick.Joystick(event.device_index)
            joy.init()
        elif event.type == pygame.JOYHATMOTION:
            # log.debug("d-pad")

            # controller.purge_dpad()
            match event.value:
                case (0, 0):
                    controller.purge_dpad()
                    if last_move != event.value:
                        post(
                            url=api_url,
                            data={"command": 'base -c {"T":1,"L":0.0,"R":0.0}'},
                        )
                        log.debug("reset")
                case (_, 0):
                    controller.release((0, 1))
                    controller.release((0, -1))
                    # post(url=api_url, data={"command": {"T": 1, "L": 0.0, "R": 0.0}})
                    log.debug("reset")
                case (0, _):
                    controller.release((1, 0))
                    controller.release((-1, 0))
                    # post(url=api_url, data={"command": {"T": 1, "L": 0.0, "R": 0.0}})
                    log.debug("reset")

            match event.value:
                case (_, 1):
                    controller.press((0, 1))
                    # cursor.up()
                    # log.debug("up")
                    cmd = {"T": 1, "L": 0.2, "R": 0.2}
                    print(
                        post(
                            url=api_url,
                            data={"command": f"base -c {dumps(cmd).replace(" ", "")}"},
                        ).content
                    )
                    log.debug("forward")
                case (_, -1):
                    controller.press((0, -1))
                    # cursor.down()
                    # log.debug("down")
                    cmd = {"T": 1, "L": -0.2, "R": -0.2}
                    post(
                        url=api_url,
                        data={"command": f"base -c {dumps(cmd).replace(" ", "")}"},
                    )
                    log.debug("back")

            match event.value:
                case (-1, _):
                    controller.press((-1, 0))
                    # cursor.left()
                    # log.debug("left")
                    cmd = {"T": 1, "L": -0.1, "R": 0.1}
                    # post(url=api_url, data={"command": })
                    post(
                        url=api_url,
                        data={"command": f"base -c {dumps(cmd).replace(" ", "")}"},
                    )
                    log.debug("left")
                case (1, _):
                    controller.press((1, 0))
                    # cursor.right()
                    # log.debug("right")
                    cmd = {"T": 1, "L": 0.1, "R": -0.1}
                    # post(url=api_url, data={"command": })
                    post(
                        url=api_url,
                        data={"command": f"base -c {dumps(cmd).replace(" ", "")}"},
                    )
                    log.debug("right")

            last_move = event.value
            # log.debug(f"got event {event}")
        elif event.type == pygame.JOYBUTTONUP:
            # log.debug(f"got event {event}")
            controller.release(event.button)
        elif event.type == pygame.JOYBUTTONDOWN:
            controller.press(event.button)
        elif (
            event.type == pygame.JOYAXISMOTION and event.axis == 4 and event.value > 0.0
        ):
            controller.press(LT)
        elif (
            event.type == pygame.JOYAXISMOTION and event.axis == 5 and event.value > 0.0
        ):
            controller.press(RT)
        elif (
            event.type == pygame.JOYAXISMOTION and event.axis == 4 and event.value < 0.0
        ):
            controller.press(LT)
        elif (
            event.type == pygame.JOYAXISMOTION and event.axis == 5 and event.value < 0.0
        ):
            controller.press(RT)

    if controller.is_pressed(HOME) and controller.just_released(START):
        break

    ret, frame = cap.read()
    if not ret:
        print("Error: Failed to retrieve frame.")
        break

    # Display the frame
    image_data = frame.data
    height, width, _ = frame.shape
    # print(frame.shape)

    # Create a Pygame surface from the received image data
    image_surface = pygame.image.frombuffer(image_data, (width, height), "BGR")
    image_surface.set_alpha(None)
    image_rect = image_surface.get_rect()
    image_rect.center = screen.get_rect().center
    fps_text = font.render(
        # Render in white
        f"FPS: {int(current_fps)}",
        True,
        (254, 255, 255),
    )
    screen.fill((0, 0, 0))
    screen.blit(image_surface, image_rect)
    screen.blit(fps_text, (10, 10))  # Position the text at (10, 10)

    pygame.display.flip()
    clock.tick()


# Release the video capture object and close all OpenCV windows
cap.release()
cv2.destroyAllWindows()
pygame.quit()
