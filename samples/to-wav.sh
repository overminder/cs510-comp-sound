for f in *.aiff; do
  ffmpeg -i "$f" "${f%.aiff}.wav"
done
