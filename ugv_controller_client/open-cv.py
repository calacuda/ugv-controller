import cv2
import pygame

# The RTSP URL (replace with your stream's URL)
# A typical format is 'rtsp://username:password@ip_address:port/stream_path'
# rtsp_url = "rtsp://your_stream_url"
# rtsp_url = "rtsp://localhost:8554/main"
rtsp_url = "rtsp://192.168.1.146:8554/main"
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
# width, height = 1280, 720
# width, height = 1920, 1080
width, height = 640, 480
screen = pygame.display.set_mode((width, height))
pygame.display.set_caption("Rendered to Pygame")
font = pygame.font.SysFont("Arial", 24)
clock = pygame.time.Clock()
fps_text = "FPS = ?"

running = True


# Read and display frames in a loop
while True:
    current_fps = clock.get_fps()

    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            break

    ret, frame = cap.read()
    if not ret:
        print("Error: Failed to retrieve frame.")
        break

    # Display the frame
    image_data = frame.data

    # Create a Pygame surface from the received image data
    image_surface = pygame.image.frombuffer(image_data, (width, height), "BGR")
    image_surface.set_alpha(None)
    fps_text = font.render(
        # Render in white
        f"FPS: {int(current_fps)}",
        True,
        (254, 255, 255),
    )
    screen.blit(image_surface, (0, 0))
    screen.blit(fps_text, (10, 10))  # Position the text at (10, 10)

    pygame.display.flip()
    clock.tick()


# Release the video capture object and close all OpenCV windows
cap.release()
cv2.destroyAllWindows()
