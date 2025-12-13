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

def create_banner(output_path, logo_path=None):
    """Create banner.bmp (493 × 58 pixels)"""
    width, height = 493, 58
    img = Image.new('RGB', (width, height), hex_to_rgb(COLORS['dark']))
    draw = ImageDraw.Draw(img)
    
    # Create subtle diagonal gradient matching dialog style
    for y in range(height):
        for x in range(width):
            # Diagonal gradient factor
            factor = (x + y) / (width + height)
            factor = factor * 0.12  # Very subtle
            
            # Interpolate between dark and dark_lighter
            r = int(hex_to_rgb(COLORS['dark'])[0] * (1 - factor) + hex_to_rgb(COLORS['dark_lighter'])[0] * factor)
            g = int(hex_to_rgb(COLORS['dark'])[1] * (1 - factor) + hex_to_rgb(COLORS['dark_lighter'])[1] * factor)
            b = int(hex_to_rgb(COLORS['dark'])[2] * (1 - factor) + hex_to_rgb(COLORS['dark_lighter'])[2] * factor)
            
            img.putpixel((x, y), (r, g, b))
    
    # Add accent line at bottom matching dialog style
    accent_height = 3
    draw.rectangle(
        [(0, height - accent_height), (width, height)],
        fill=hex_to_rgb(COLORS['primary'])
    )
    
    # Try to add text
    try:
        # Attempt to use a nice font
        font = ImageFont.truetype("arial.ttf", 24)
        font_small = ImageFont.truetype("arial.ttf", 14)
    except:
        # Fallback to default font
        font = ImageFont.load_default()
        font_small = ImageFont.load_default()
    
    # Add title text
    title = "KABLE"
    subtitle = "Minecraft Launcher"
    
    # Add logo if available
    logo_x = 25
    text_x_offset = 0
    
    if logo_path and os.path.exists(logo_path):
        try:
            logo = Image.open(logo_path)
            # Resize logo to fit banner height (leaving some padding)
            logo_height = height - 16  # 8px padding top and bottom
            aspect_ratio = logo.width / logo.height
            logo_width = int(logo_height * aspect_ratio)
            logo = logo.resize((logo_width, logo_height), Image.Resampling.LANCZOS)
            
            # Convert to RGBA if not already
            if logo.mode != 'RGBA':
                logo = logo.convert('RGBA')
            
            # Paste logo on the left side
            logo_y = (height - logo_height) // 2
            img.paste(logo, (logo_x, logo_y), logo)
            
            # Adjust text position to be after logo
            text_x_offset = logo_width + 15
        except Exception as e:
            print(f"Warning: Could not load logo: {e}")
    
    x = logo_x + text_x_offset
    
    # Calculate title height for vertical centering
    title_bbox = draw.textbbox((0, 0), title, font=font)
    title_width = title_bbox[2] - title_bbox[0]
    title_height = title_bbox[3] - title_bbox[1]
    
    subtitle_bbox = draw.textbbox((0, 0), subtitle, font=font_small)
    subtitle_height = subtitle_bbox[3] - subtitle_bbox[1]
    
    # Calculate vertical position - slightly above center for better visual balance
    total_text_height = title_height + 2
    y_title = (height - total_text_height) // 2 - 4  # Shift up 4px
    
    # Draw title with darker purple shadow
    shadow_offset = 2
    # Darker purple shadow (reduce RGB values by ~40%)
    primary_rgb = hex_to_rgb(COLORS['primary'])
    dark_purple = tuple(int(c * 0.5) for c in primary_rgb)
    draw.text((x + shadow_offset, y_title + shadow_offset), title, 
              fill=dark_purple, font=font)
    # Main text in magenta/pink
    draw.text((x, y_title), title, fill=hex_to_rgb(COLORS['secondary']), font=font)
    
    # Draw subtitle (vertically centered with title)
    subtitle_y = y_title + (title_height - subtitle_height) // 2
    draw.text((x + title_width + 15, subtitle_y), subtitle, 
              fill=hex_to_rgb(COLORS['text']), font=font_small)
    
    # Add small decorative squares on the right side matching dialog
    square_size = 2
    spacing = 10
    start_x = width - 100
    start_y = 15
    
    for row in range(3):
        for col in range(6):
            square_x = start_x + col * spacing
            square_y = start_y + row * spacing
            # Alternate colors for pattern
            if (row + col) % 2 == 0:
                color = hex_to_rgb(COLORS['primary'])
            else:
                color = hex_to_rgb(COLORS['tertiary'])
            
            # Make squares semi-transparent by reducing intensity
            color = tuple(int(c * 0.3) for c in color)
            draw.rectangle(
                [(square_x, square_y), (square_x + square_size, square_y + square_size)],
                fill=color
            )
    
    # Save as BMP
    img.save(output_path, 'BMP')
    print(f"✓ Created banner: {output_path}")

def create_dialog(output_path, logo_path=None):
    """Create dialog.bmp (493 × 312 pixels) - horizontal dialog background"""
    width, height = 493, 312
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
    
    # Add "Kable" branding text horizontally in center
    try:
        # Try to use Arial
        font_large = ImageFont.truetype("arial.ttf", 72)
        font_small = ImageFont.truetype("arial.ttf", 24)
    except:
        # Fallback to default
        font_large = ImageFont.load_default()
        font_small = ImageFont.load_default()
    
    # Add main title in center
    text = "KABLE"
    subtitle = "Minecraft Launcher"
    
    # Add logo if available
    if logo_path and os.path.exists(logo_path):
        try:
            logo = Image.open(logo_path)
            # Resize logo to reasonable size
            logo_height = 120
            aspect_ratio = logo.width / logo.height
            logo_width = int(logo_height * aspect_ratio)
            logo = logo.resize((logo_width, logo_height), Image.Resampling.LANCZOS)
            
            # Convert to RGBA if not already
            if logo.mode != 'RGBA':
                logo = logo.convert('RGBA')
            
            # Center logo horizontally, position in upper third
            logo_x = (width - logo_width) // 2
            logo_y = height // 4 - logo_height // 2
            img.paste(logo, (logo_x, logo_y), logo)
        except Exception as e:
            print(f"Warning: Could not load logo: {e}")
    
    # Calculate text position for centering
    title_bbox = draw.textbbox((0, 0), text, font=font_large)
    title_width = title_bbox[2] - title_bbox[0]
    title_height = title_bbox[3] - title_bbox[1]
    
    subtitle_bbox = draw.textbbox((0, 0), subtitle, font=font_small)
    subtitle_width = subtitle_bbox[2] - subtitle_bbox[0]
    
    # Center the text in the lower half
    title_x = (width - title_width) // 2
    title_y = height * 2 // 3 - 20
    
    subtitle_x = (width - subtitle_width) // 2
    subtitle_y = title_y + title_height + 15
    
    # Draw title with subtle shadow
    shadow_offset = 2
    draw.text((title_x + shadow_offset, title_y + shadow_offset), text, 
              fill=(0, 0, 0), font=font_large)
    draw.text((title_x, title_y), text, 
              fill=hex_to_rgb(COLORS['primary']), font=font_large)
    
    # Draw subtitle
    draw.text((subtitle_x, subtitle_y), subtitle, 
              fill=hex_to_rgb(COLORS['text']), font=font_small)
    
    # Add subtle decorative elements
    # Small squares pattern in bottom corners
    square_size = 2
    spacing = 12
    
    # Bottom left
    start_x_left = 20
    start_y = height - 60
    
    for row in range(4):
        for col in range(6):
            x = start_x_left + col * spacing
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
    
    # Bottom right mirror pattern
    start_x_right = width - 90
    for row in range(4):
        for col in range(6):
            x = start_x_right + col * spacing
            y = start_y + row * spacing
            if (row + col) % 2 == 0:
                color = hex_to_rgb(COLORS['secondary'])
            else:
                color = hex_to_rgb(COLORS['tertiary'])
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
    project_root = os.path.join(script_dir, '..')
    wix_dir = os.path.join(project_root, 'src-tauri', 'wix')
    img_dir = os.path.join(wix_dir, 'img')
    
    # Try to find logo
    logo_path = os.path.join(project_root, 'static', 'favicon.png')
    if not os.path.exists(logo_path):
        # Fallback to icon.png
        logo_path = os.path.join(project_root, 'src-tauri', 'icons', 'icon.png')
    if not os.path.exists(logo_path):
        logo_path = None
        print("Warning: Logo not found, generating without logo")
    
    # Create img directory if it doesn't exist
    os.makedirs(img_dir, exist_ok=True)
    
    banner_path = os.path.join(img_dir, 'banner.bmp')
    dialog_path = os.path.join(img_dir, 'dialog.bmp')
    
    print("Generating WiX installer images...")
    print(f"Output directory: {wix_dir}")
    if logo_path:
        print(f"Using logo: {logo_path}")
    print()
    
    create_banner(banner_path, logo_path)
    create_dialog(dialog_path, logo_path)
    
    print()
    print("✓ All images generated successfully!")
    print()
    print("Theme colors used:")
    for name, color in COLORS.items():
        print(f"  {name}: {color}")

if __name__ == '__main__':
    main()
