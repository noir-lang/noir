import { useState, useEffect, useRef } from 'react';
import classnames from 'classnames';

import { DropdownOption, DropdownType, Dropdown } from './dropdown.js';
import style from './select.module.scss';

interface SelectProps<T> {
  options: DropdownOption<T>[];
  dropdownType?: DropdownType;
  showBorder?: boolean;
  allowEmptyValue?: boolean;
  disabled?: boolean;
  placeholder?: string;
  className?: string;
  value?: T;
  onChange?: (value?: T) => void;
}

function useOutsideAlerter(ref: any, setIsOpen: any) {
  useEffect(() => {
    function handleClickOutside(event: { target: any }) {
      if (ref.current && !ref.current.contains(event.target)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [ref, setIsOpen]);
}

export function Select<T>(props: SelectProps<T>) {
  const { showBorder = true } = props;
  const [isOpen, setIsOpen] = useState(false);
  const wrapperRef = useRef(null);

  useOutsideAlerter(wrapperRef, setIsOpen);

  useEffect(() => {
    setIsOpen(false);
  }, [props.value]);

  const handleTriggerDropdown = () => {
    if (props.disabled) return;
    setIsOpen(prevValue => !prevValue);
  };

  const handleOptionSelect = (option: DropdownOption<T>) => {
    handleChange(option.value);
    setIsOpen(false);
  };

  const handleChange = (value?: T) => {
    if (props.onChange) {
      props.onChange(value);
    }
  };

  const hasButton = props.value && props.allowEmptyValue;
  const activeLabel = props.options.find(x => x.value === props.value)?.label;

  return (
    <div
      ref={wrapperRef}
      className={classnames(
        style.selectBox,
        showBorder && style.border,
        props.disabled && style.disabled,
        props.className,
      )}
      onClick={handleTriggerDropdown}
    >
      <div className={classnames(style.innerFrame, hasButton && style.innerFrameWithButton)}>
        <span className={classnames(style.value, !activeLabel && style.placeholder)}>
          {activeLabel || props.placeholder}
        </span>
        <Dropdown className={style.dropdown} isOpen={isOpen} options={props.options} onClick={handleOptionSelect} />
      </div>
    </div>
  );
}
