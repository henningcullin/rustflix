import * as React from 'react';

interface SaveCartridgeIconProps extends React.SVGProps<SVGSVGElement> {
  title?: string;
}

const SaveCartridgeIcon = React.forwardRef<
  SVGSVGElement,
  SaveCartridgeIconProps
>(function SaveCartridgeIcon({ title = 'Save Cartridge Icon', ...props }, ref) {
  return (
    <svg
      viewBox='0 0 24 24'
      fill='none' // Set fill to none to emphasize the outline style
      stroke='currentColor' // Stroke color will be used for outlines
      strokeWidth='1.5' // Thinner stroke for a lighter outline
      height={props.height || '1em'}
      width={props.width || '1em'}
      ref={ref}
      aria-hidden={title ? 'false' : 'true'}
      role={title ? 'img' : 'presentation'}
      {...props}
    >
      {title && <title>{title}</title>}
      <path
        d='M3.75 2A1.75 1.75 0 002 3.75v16.5A1.75 1.75 0 003.75 22h16.5A1.75 1.75 0 0022 20.25V7.56c0-.465-.185-.912-.513-1.24l-3.807-3.807A1.75 1.75 0 0015.44 2H3.75z'
        strokeLinecap='round'
        strokeLinejoin='round'
      />
      <rect
        x='7.25'
        y='10'
        width='9'
        height='7.5'
        rx='0.75'
        ry='0.75'
        strokeLinecap='round'
        strokeLinejoin='round'
      />
      <path d='M10 4.75v3.5' strokeLinecap='round' strokeLinejoin='round' />
      <path
        d='M17.5 10h-7v4.5h7V10z'
        fill='none'
        strokeLinecap='round'
        strokeLinejoin='round'
      />
    </svg>
  );
});

SaveCartridgeIcon.displayName = 'SaveCartridgeIcon';

export default SaveCartridgeIcon;
