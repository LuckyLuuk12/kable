"""
Generate WiX Installer Banner and Dialog Images
Creates professionally styled images for the Kable MSI installer
with the project's theme colors.

Requirements: pip install pillow
Usage: python generate-wix-images.py
"""

from PIL import Image, ImageDraw, ImageFont
import os

# Theme colors from global.scss
COLORS = {
    'dark': '#0a0a0f',
    'dark_lighter': '#1e1e20',
    'dark_card': '#26262a',
    'primary': '#8b5cf6',
    'secondary': '#ec4899',
    'tertiary': '#0ea5e9',
    'text': '#ffffff',
    'placeholder': '#a0a0aa'
}

def hex_to_rgb(hex_color):
    """Convert hex color to RGB tuple"""
    hex_color = hex_color.lstrip('#')
    return tuple(int(hex_color[i:i+2], 16) for i in (0, 2, 4))

def create_banner(output_path):
    """Create banner.bmp (493 × 58 pixels)"""
    width, height = 493, 58
    img = Image.new('RGB', (width, height), hex_to_rgb(COLORS['dark']))
    draw = ImageDraw.Draw(img)
    
    # Create gradient effect with primary color
    for y in range(height):
        alpha = y / height
        r = int(hex_to_rgb(COLORS['dark'])[0] * (1 - alpha) + hex_to_rgb(COLORS['primary'])[0] * alpha * 0.3)
        g = int(hex_to_rgb(COLORS['dark'])[1] * (1 - alpha) + hex_to_rgb(COLORS['primary'])[1] * alpha * 0.3)
        b = int(hex_to_rgb(COLORS['dark'])[2] * (1 - alpha) + hex_to_rgb(COLORS['primary'])[2] * alpha * 0.3)
        draw.line([(0, y), (width, y)], fill=(r, g, b))
    
    # Add accent line at bottom
    accent_height = 3
    draw.rectangle(
        [(0, height - accent_height), (width, height)],
        fill=hex_to_rgb(COLORS['primary'])
    )
    
    # Try to add text
    try:
        # Attempt to use a nice font
        font = ImageFont.truetype("arial.ttf", 20)
        font_small = ImageFont.truetype("arial.ttf", 12)
    except:
        # Fallback to default font
        font = ImageFont.load_default()
        font_small = ImageFont.load_default()
    
    # Add title text
    title = "Kable"
    subtitle = "Minecraft Launcher"
    
    # Calculate text position
    title_bbox = draw.textbbox((0, 0), title, font=font)
    title_width = title_bbox[2] - title_bbox[0]
    
    x = 20
    y_title = 10
    
    # Draw title
    draw.text((x, y_title), title, fill=hex_to_rgb(COLORS['text']), font=font)
    
    # Draw subtitle
    draw.text((x + title_width + 10, y_title + 5), subtitle, 
              fill=hex_to_rgb(COLORS['placeholder']), font=font_small)
    
    # Save as BMP
    img.save(output_path, 'BMP')
    print(f"✓ Created banner: {output_path}")

def create_dialog(output_path):
    """Create dialog.bmp (164 × 314 pixels) - vertical left sidebar"""
    width, height = 164, 314
    img = Image.new('RGB', (width, height), hex_to_rgb(COLORS['dark']))
    draw = ImageDraw.Draw(img)
    
    # Create subtle diagonal gradient from dark to slightly lighter
    for y in range(height):
        for x in range(width):
            # Diagonal gradient factor
            factor = (x + y) / (width + height)
            factor = factor * 0.15  # Very subtle (only 15% lighter at most)
            
            # Interpolate between dark and dark_lighter
            r = int(hex_to_rgb(COLORS['dark'])[0] * (1 - factor) + hex_to_rgb(COLORS['dark_lighter'])[0] * factor)
            g = int(hex_to_rgb(COLORS['dark'])[1] * (1 - factor) + hex_to_rgb(COLORS['dark_lighter'])[1] * factor)
            b = int(hex_to_rgb(COLORS['dark'])[2] * (1 - factor) + hex_to_rgb(COLORS['dark_lighter'])[2] * factor)
            
            img.putpixel((x, y), (r, g, b))
    
    # Add subtle geometric pattern overlay
    # Thin vertical accent line on the right edge
    accent_width = 3
    draw.rectangle(
        [(width - accent_width, 0), (width, height)],
        fill=hex_to_rgb(COLORS['primary'])
    )
    
    # Add "Kable" branding text vertically along the right side
    try:
        # Try to use Arial
        font_large = ImageFont.truetype("arial.ttf", 48)
        font_small = ImageFont.truetype("arial.ttf", 14)
    except:
        # Fallback to default
        font_large = ImageFont.load_default()
        font_small = ImageFont.load_default()
    
    # Create text image for rotation
    text = "KABLE"
    text_img = Image.new('RGBA', (300, 100), (0, 0, 0, 0))
    text_draw = ImageDraw.Draw(text_img)
    text_draw.text((10, 10), text, fill=hex_to_rgb(COLORS['primary']) + (40,), font=font_large)
    
    # Rotate text 90 degrees
    text_img = text_img.rotate(90, expand=True)
    
    # Paste text onto main image (positioned on right side, centered vertically)
    text_width, text_height = text_img.size
    paste_x = width - 60
    paste_y = (height - text_height) // 2
    img.paste(text_img, (paste_x, paste_y), text_img)
    
    # Add subtle decorative elements
    # Small squares pattern in bottom left
    square_size = 2
    spacing = 12
    start_x = 20
    start_y = height - 60
    
    for row in range(4):
        for col in range(6):
            x = start_x + col * spacing
            y = start_y + row * spacing
            # Alternate colors for pattern
            if (row + col) % 2 == 0:
                color = hex_to_rgb(COLORS['primary'])
            else:
                color = hex_to_rgb(COLORS['tertiary'])
            
            # Make squares semi-transparent by reducing intensity
            color = tuple(int(c * 0.3) for c in color)
            draw.rectangle(
                [(x, y), (x + square_size, y + square_size)],
                fill=color
            )
    
    # Add subtle corner accent in top left
    corner_size = 80
    for i in range(corner_size):
        alpha = 1 - (i / corner_size)
        alpha = alpha * 0.2  # Very subtle
        color_val = int(hex_to_rgb(COLORS['primary'])[0] * alpha)
        color = (color_val, int(hex_to_rgb(COLORS['primary'])[1] * alpha), 
                 int(hex_to_rgb(COLORS['primary'])[2] * alpha))
        
        # Draw diagonal line
        for j in range(i):
            x = j
            y = i - j
            if x < width and y < height:
                current = img.getpixel((x, y))
                blended = tuple(min(255, current[k] + color[k]) for k in range(3))
                img.putpixel((x, y), blended)
    
    # Save as BMP
    img.save(output_path, 'BMP')
    print(f"✓ Created dialog: {output_path}")

def main():
    """Generate both WiX installer images"""
    script_dir = os.path.dirname(os.path.abspath(__file__))
    wix_dir = os.path.join(script_dir, '..', 'src-tauri', 'wix')
    
    # Create wix directory if it doesn't exist
    os.makedirs(wix_dir, exist_ok=True)
    
    banner_path = os.path.join(wix_dir, 'banner.bmp')
    dialog_path = os.path.join(wix_dir, 'dialog.bmp')
    
    print("Generating WiX installer images...")
    print(f"Output directory: {wix_dir}")
    print()
    
    create_banner(banner_path)
    create_dialog(dialog_path)
    
    print()
    print("✓ All images generated successfully!")
    print()
    print("Theme colors used:")
    for name, color in COLORS.items():
        print(f"  {name}: {color}")

if __name__ == '__main__':
    main()
