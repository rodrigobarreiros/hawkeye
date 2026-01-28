FROM bluenviron/mediamtx:latest

# Copy custom configuration
COPY mediamtx.yml /mediamtx.yml

# Copy web files for HLS/WebRTC player
COPY web/ /web/

EXPOSE 8554 8555 8888 8889 9000 9997 9998

ENTRYPOINT ["/mediamtx"]
CMD ["/mediamtx.yml"]
