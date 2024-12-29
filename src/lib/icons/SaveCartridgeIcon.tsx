import * as React from 'react';

const SaveCartridgeIcon = React.forwardRef<
  SVGSVGElement,
  React.SVGProps<SVGSVGElement>
>(({ className, ...props }, ref) => {
  return (
    <svg
      viewBox='0 0 24 24'
      fill='currentColor'
      height='1em'
      width='1em'
      ref={ref}
      className={className} // Make sure className is passed to apply styles
      {...props} // Spread remaining props (e.g., onClick, role)
    >
      <path
        fillRule='evenodd'
        d='M3.75 2A1.75 1.75 0 002 3.75v16.5A1.75 1.75 0 003.75 22h16.5A1.75 1.75 0 0022 20.25V7.56c0-.465-.185-.912-.513-1.24l-3.807-3.807A1.75 1.75 0 0015.44 2H3.75zM3.5 3.75c0-.138.112-.25.25-.25h11.69c.066 0 .13.026.177.073l3.807 3.807a.25.25 0 01.073.177v12.693a.25.25 0 01-.25.25H3.75a.25.25 0 01-.25-.25V3.75zm14 7.5a.75.75 0 00-.75-.75h-8.5a.75.75 0 00-.75.75v6a.75.75 0 00.75.75h8.5a.75.75 0 00.75-.75v-6zm-8 1.5h7v4.5h-7v-4.5zm3-8a.5.5 0 00-1 0v3.5h1V4.75z'
      />
    </svg>
  );
});

SaveCartridgeIcon.displayName = 'SaveCartridgeIcon';

export default SaveCartridgeIcon;
