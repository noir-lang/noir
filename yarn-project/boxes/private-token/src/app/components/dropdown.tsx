import { CSSProperties, useEffect, useRef } from 'react';
import classnames from 'classnames';

import style from './dropdown.module.scss';

export enum DropdownType {
  Simple = 'Simple',
  Fees = 'Fees',
}

function useOutsideAlerter(ref: any, cb: any) {
  useEffect(() => {
    /**
     * Alert if clicked on outside of element
     */
    function handleClickOutside(event: any) {
      if (ref.current && !ref.current.contains(event.target)) {
        cb('You clicked outside of me!');
      }
    }

    // Bind the event listener
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      // Unbind the event listener on clean up
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [ref, cb]);
}

interface DropdownProps<T> {
  options: DropdownOption<T>[];
  isOpen?: boolean;
  className?: string;
  style?: CSSProperties;
  onClose?: () => void;
  onClick?: (option: DropdownOption<T>) => void;
}

export interface DropdownOption<T> {
  value: T;
  label: string;
  sublabel?: string;
  image?: string;
  disabled?: boolean;
}

export function Dropdown<T>(props: DropdownProps<T>) {
  const wrapperRef = useRef(null);
  useOutsideAlerter(wrapperRef, () => props.onClose && props.onClose());

  if (!props.isOpen) {
    return null;
  }

  const handleClick = (option: DropdownOption<T>) => {
    if (option.disabled) {
      return;
    }
    if (props.onClick) {
      props.onClick(option);
    }
    if (props.onClose) {
      props.onClose();
    }
  };

  return (
    <div ref={wrapperRef} style={props.style} className={classnames(style.dropdownWrapper, props.className)}>
      {props.options.map((option: DropdownOption<T>) => (
        <div
          className={classnames(style.dropdownOptionBackground, option.disabled && style.disabled)}
          onClick={() => handleClick(option)}
          key={option.label}
        >
          <div className={classnames(style.dropdownOption, style.singleOption, option.disabled && style.disabled)}>
            {option.image && <img src={option.image} alt={option.label} />}
            <div className={style.label}>{option.label}</div>
          </div>
        </div>
      ))}
    </div>
  );
}
