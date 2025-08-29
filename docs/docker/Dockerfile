# Use nginx:alpine for a lightweight image
FROM nginx:alpine

# Remove default nginx configuration
RUN rm /etc/nginx/nginx.conf

# Copy our nginx configuration
COPY docker/nginx.conf /etc/nginx/nginx.conf

# Copy the built Docusaurus site
COPY build/ /usr/share/nginx/html/

# Create nginx directories and set permissions
RUN mkdir -p /var/log/nginx && \
    touch /var/log/nginx/access.log /var/log/nginx/error.log && \
    chmod 644 /var/log/nginx/access.log /var/log/nginx/error.log

# Expose port 80
EXPOSE 80

# Start nginx in foreground
CMD ["nginx", "-g", "daemon off;"]